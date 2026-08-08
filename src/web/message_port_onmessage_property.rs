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
    let Some(record) = scope
        .get_slot::<super::message_port::MessagePortStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.onmessage.as_ref() {
        Some(handler) => result.set(v8::Local::new(scope, handler)),
        None => result.set(v8::null(scope).into()),
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let handler = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|function| v8::Global::new(scope, v8::Local::<v8::Value>::from(function)));
    let Some(record) = scope
        .get_slot_mut::<super::message_port::MessagePortStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.onmessage = handler;
    record.started = true;
    super::message_port::schedule_delivery(scope, id);
}
