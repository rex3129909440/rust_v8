pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "postMessage", 1, post_message)
}

fn post_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'postMessage': 1 argument required",
        );
        return;
    }
    let sender_id = arguments.this().get_identity_hash().get();
    let Some(sender) = scope
        .get_slot::<super::broadcast_channel::BroadcastChannelStore>()
        .and_then(|store| store.records.get(&sender_id))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if sender.closed {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "Channel is closed".to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let recipients = scope
        .get_slot::<super::broadcast_channel::BroadcastChannelStore>()
        .map(|store| {
            let mut recipients = store
                .records
                .iter()
                .filter(|(id, record)| {
                    **id != sender_id
                        && !record.closed
                        && record.name == sender.name
                        && record.origin == sender.origin
                })
                .map(|(id, record)| (*id, record.context.clone()))
                .collect::<Vec<_>>();
            recipients.sort_by_key(|(id, _)| *id);
            recipients
        })
        .unwrap_or_default();

    for (recipient_id, context) in recipients {
        let context_local = v8::Local::new(scope, &context);
        let cloned = match super::structured_clone::clone_into(
            scope,
            context_local,
            arguments.get(0),
            super::structured_clone::TransferList::default(),
        ) {
            Ok(cloned) => cloned,
            Err(message) => {
                super::structured_clone::throw_data_clone_error(scope, &message);
                return;
            }
        };
        if let Some(store) = scope.get_slot_mut::<super::broadcast_channel::BroadcastChannelStore>()
        {
            store
                .pending
                .push_back(super::broadcast_channel::PendingBroadcastMessage {
                    recipient_id,
                    data: cloned.value,
                    origin: sender.origin.clone(),
                });
        }
        if let Some(task) = v8::Function::new(scope, dispatch_next) {
            scope.enqueue_microtask(task);
        }
    }
}

fn dispatch_next(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let message = scope
        .get_slot_mut::<super::broadcast_channel::BroadcastChannelStore>()
        .and_then(|store| store.pending.pop_front());
    let Some(message) = message else {
        return;
    };
    let recipient = scope
        .get_slot::<super::broadcast_channel::BroadcastChannelStore>()
        .and_then(|store| store.records.get(&message.recipient_id))
        .cloned();
    let Some(recipient) = recipient.filter(|recipient| !recipient.closed) else {
        return;
    };
    let context = v8::Local::new(scope, &recipient.context);
    let target_scope = &mut v8::ContextScope::new(scope, context);
    let target = v8::Local::new(target_scope, &recipient.object);
    let data = v8::Local::new(target_scope, &message.data);
    let Ok(event) = super::message_event::create(
        target_scope,
        "message",
        data,
        &message.origin,
        None,
        Vec::new(),
    ) else {
        return;
    };
    if let Some(handler) = recipient.onmessage {
        let handler = v8::Local::new(target_scope, &handler);
        let _ = handler.call(target_scope, target.into(), &[event.into()]);
    }
    let _ = super::event_target::dispatch(target_scope, target, event);
}
