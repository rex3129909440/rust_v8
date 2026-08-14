pub(crate) fn define_for_current_realm(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "MediaSource")?;
    let constructor = global
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "Worker MediaSource constructor is unavailable".to_owned())?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "handle", get_handle)
}

fn get_handle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::media_source::handle(scope, arguments.this()) {
        Ok(handle) => result.set(v8::Local::new(scope, &handle).into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
