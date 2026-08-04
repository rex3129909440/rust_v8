pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "resizeBy",
        2,
        v8::ConstructorBehavior::Throw,
        resize_by,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "resizeBy")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.resizeBy".to_owned())
    }
}

fn resize_by(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = arguments.get(0).number_value(scope).unwrap_or(0.0);
    let height = arguments.get(1).number_value(scope).unwrap_or(0.0);
    super::window_view_state::resize_by(scope, width, height);
}
