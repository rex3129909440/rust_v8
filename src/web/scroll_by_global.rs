pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "scrollBy",
        0,
        v8::ConstructorBehavior::Throw,
        scroll_by,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "scrollBy")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.scrollBy".to_owned())
    }
}

fn scroll_by(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let (x, y) = if let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        (
            number_property(scope, options, "left").unwrap_or(0.0),
            number_property(scope, options, "top").unwrap_or(0.0),
        )
    } else {
        (
            arguments.get(0).number_value(scope).unwrap_or(0.0),
            arguments.get(1).number_value(scope).unwrap_or(0.0),
        )
    };
    super::window_view_state::scroll_by(scope, x, y);
    let global = scope.get_current_context().global(scope);
    let event = super::event_target::create_event(scope, "scroll");
    super::event_target::dispatch(scope, global, event);
}

fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined())
        .then(|| value.number_value(scope))
        .flatten()
}
