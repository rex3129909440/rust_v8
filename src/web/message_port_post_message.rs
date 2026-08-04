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
    let Some((peer_id, sender_closed, sender_detached)) = scope
        .get_slot::<super::message_port::MessagePortStore>()
        .and_then(|store| store.records.get(&sender_id))
        .map(|record| (record.peer, record.closed, record.detached))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if sender_detached {
        super::structured_clone::throw_data_clone_error(scope, "The MessagePort is detached.");
        return;
    }
    if sender_closed {
        return;
    }
    let Some(peer_id) = peer_id else {
        return;
    };
    let peer_context = scope
        .get_slot::<super::message_port::MessagePortStore>()
        .and_then(|store| store.records.get(&peer_id))
        .map(|record| record.context.clone());
    let Some(peer_context) = peer_context else {
        return;
    };
    let transfer = if arguments.get(1).is_array() {
        super::structured_clone::transfer_from_sequence(scope, arguments.get(1))
    } else {
        super::structured_clone::transfer_from_options(scope, arguments.get(1))
    };
    let transfer = match transfer {
        Ok(transfer) => transfer,
        Err(message) => {
            super::structured_clone::throw_data_clone_error(scope, &message);
            return;
        }
    };
    if transfer.contains_message_port(sender_id) {
        super::structured_clone::throw_data_clone_error(
            scope,
            "A MessagePort cannot transfer itself from its own postMessage call.",
        );
        return;
    }
    let peer_context = v8::Local::new(scope, &peer_context);
    let cloned = match super::structured_clone::clone_into(
        scope,
        peer_context,
        arguments.get(0),
        transfer,
    ) {
        Ok(cloned) => cloned,
        Err(message) => {
            super::structured_clone::throw_data_clone_error(scope, &message);
            return;
        }
    };
    let queued = super::message_port::QueuedMessage {
        data: cloned.value,
        ports: cloned.ports,
    };
    let deliverable = scope
        .get_slot_mut::<super::message_port::MessagePortStore>()
        .and_then(|store| store.records.get_mut(&peer_id))
        .is_some_and(|peer| {
            if peer.closed {
                false
            } else {
                peer.pending.push(queued);
                true
            }
        });
    if deliverable {
        super::message_port::schedule_delivery(scope, peer_id);
    }
}
