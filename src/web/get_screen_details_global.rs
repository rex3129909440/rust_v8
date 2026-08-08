pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "getScreenDetails",
        0,
        v8::ConstructorBehavior::Throw,
        get_screen_details,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "getScreenDetails")?;
    match global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot define window.getScreenDetails".to_owned()),
    }
}

fn get_screen_details(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::screen_details::create(scope) {
        Ok(details) => {
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, details.into()) {
                result.set(promise.into());
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
