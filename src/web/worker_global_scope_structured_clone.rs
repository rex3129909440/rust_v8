pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "structuredClone", 1, call)
}
fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'structuredClone' on 'WorkerGlobalScope': 1 argument required, but only 0 present.",
        );
        return;
    }
    let context = v8::Global::new(scope, scope.get_entered_or_microtask_context());
    let transfer = match super::structured_clone::transfer_from_options(scope, arguments.get(1)) {
        Ok(transfer) => transfer,
        Err(message) => {
            super::structured_clone::throw_data_clone_error(scope, &message);
            return;
        }
    };
    let context = v8::Local::new(scope, &context);
    match super::structured_clone::clone_into(scope, context, arguments.get(0), transfer) {
        Ok(cloned) => result.set(v8::Local::new(scope, &cloned.value)),
        Err(message) => super::structured_clone::throw_data_clone_error(scope, &message),
    }
}
