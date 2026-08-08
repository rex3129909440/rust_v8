pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "respondWith", 1, respond_with)
}

fn respond_with(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'respondWith': 1 argument required",
        );
        return;
    }
    let id = arguments.this().get_identity_hash().get();
    let response = v8::Global::new(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<super::fetch_event::FetchEventStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.response.is_some() {
        crate::webidl::throw_type_error(scope, "respondWith has already been called");
        return;
    }
    record.response = Some(response);
}
