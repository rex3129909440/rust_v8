pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "blur", 0, v8::ConstructorBehavior::Throw, blur)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "blur")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.blur".to_owned())
    }
}
fn blur(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::window_lifecycle_state::blur(scope);
}
