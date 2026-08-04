pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "cancelAnimationFrame",
        1,
        v8::ConstructorBehavior::Throw,
        cancel_animation_frame,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "cancelAnimationFrame")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.cancelAnimationFrame".to_owned())
    }
}
fn cancel_animation_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.get(0).int32_value(scope).unwrap_or(0);
    super::animation_frame_state::cancel(scope, id);
}
