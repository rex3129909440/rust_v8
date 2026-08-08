pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "waitUntil", 1, wait_until)
}

fn wait_until(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'waitUntil': 1 argument required",
        );
        return;
    }
    let id = arguments.this().get_identity_hash().get();
    let promise = v8::Global::new(scope, arguments.get(0));
    let Some(promises) = scope
        .get_slot_mut::<super::extendable_event::ExtendableEventStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    promises.push(promise);
}
