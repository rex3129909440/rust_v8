pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "cancelIdleCallback",
        1,
        v8::ConstructorBehavior::Throw,
        cancel_idle_callback,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "cancelIdleCallback")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.cancelIdleCallback".to_owned())
    }
}

fn cancel_idle_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.get(0).int32_value(scope).unwrap_or(0);
    super::idle_callback_state::cancel(scope, id);
}
