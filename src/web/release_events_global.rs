pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "releaseEvents",
        0,
        v8::ConstructorBehavior::Throw,
        release_events,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "releaseEvents")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.releaseEvents".to_owned())
    }
}

fn release_events(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let event_mask = arguments.get(0).uint32_value(scope).unwrap_or(0);
    super::capture_events_global::release(scope, event_mask);
}
