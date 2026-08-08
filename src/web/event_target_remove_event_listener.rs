use super::event_target::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "removeEventListener",
        2,
        remove_event_listener,
    )
}

fn remove_event_listener(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let identity = v8::Local::<v8::Object>::try_from(arguments.get(1))
        .ok()
        .map(|callback| callback.get_identity_hash().get());
    let capture = capture_option(scope, arguments.get(2));
    let target_id = target_record_id(scope, arguments.this());
    let Some(record) = scope
        .get_slot_mut::<EventTargetStore>()
        .and_then(|store| store.targets.get_mut(&target_id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let (Some(identity), Some(listeners)) = (identity, record.listeners.get_mut(&event_type)) {
        listeners
            .iter()
            .position(|listener| listener.identity == identity && listener.capture == capture)
            .map(|index| listeners.remove(index));
    }
}
