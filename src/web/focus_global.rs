pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "focus", 0, v8::ConstructorBehavior::Throw, focus)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "focus")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.focus".to_owned())
    }
}

fn focus(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::window_lifecycle_state::focus(scope);
}
