use super::event_target::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "addEventListener", 2, add_event_listener)
}

fn add_event_listener(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !is_event_target(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    if arguments.get(1).is_null() || arguments.get(1).is_undefined() {
        return;
    }
    let Ok(callback) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'addEventListener' on 'EventTarget': parameter 2 is not of type 'Object'.",
        );
        return;
    };
    let Some(mut options) = listener_options(scope, arguments.get(2)) else {
        return;
    };
    if !options.passive_specified
        && matches!(
            event_type.as_str(),
            "touchstart" | "touchmove" | "wheel" | "mousewheel"
        )
        && default_passive_target(scope, arguments.this())
    {
        options.passive = true;
    }
    let identity = callback.get_identity_hash().get();
    let callback = v8::Global::new(scope, callback);
    let target_id = target_record_id(scope, arguments.this());
    let Some(store) = scope.get_slot_mut::<EventTargetStore>() else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(record) = store.targets.get_mut(&target_id) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let duplicate = record.listeners.get(&event_type).is_some_and(|listeners| {
        listeners
            .iter()
            .any(|listener| listener.identity == identity && listener.capture == options.capture)
    });
    if duplicate {
        return;
    }
    let registration_id = store.next_listener_id;
    store.next_listener_id = store.next_listener_id.wrapping_add(1).max(1);
    store
        .targets
        .get_mut(&target_id)
        .expect("EventTarget record")
        .listeners
        .entry(event_type)
        .or_default()
        .push(EventListener {
            registration_id,
            identity,
            callback,
            capture: options.capture,
            once: options.once,
            passive: options.passive,
            signal_identity: options.signal,
        });
}

fn default_passive_target(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> bool {
    super::window_event_handler_support::is_window(scope, target)
        || super::document::serialize_if_document(scope, target).is_some()
        || super::node::record(scope, target).is_some_and(|record| record.node_name == "BODY")
}
