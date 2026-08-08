pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "onmessage", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let handler = scope
        .get_slot::<super::broadcast_channel::BroadcastChannelStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.onmessage.clone());
    match handler {
        Some(Some(handler)) => result.set(v8::Local::new(scope, &handler).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|handler| v8::Global::new(scope, handler));
    let Some(record) = scope
        .get_slot_mut::<super::broadcast_channel::BroadcastChannelStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.onmessage = handler;
}
