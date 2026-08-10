pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "requestIdleCallback",
        1,
        v8::ConstructorBehavior::Throw,
        request_idle_callback,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "requestIdleCallback")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.requestIdleCallback".to_owned())
    }
}

fn request_idle_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'requestIdleCallback' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'requestIdleCallback' on 'Window': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let timeout_ms = v8::Local::<v8::Object>::try_from(arguments.get(1))
        .ok()
        .and_then(|options| {
            let key = v8::String::new(scope, "timeout")?;
            let value = options.get(scope, key.into())?;
            (!value.is_undefined())
                .then(|| value.number_value(scope))
                .flatten()
                .map(super::timer_state::normalized_delay)
        });
    let id = super::idle_callback_state::reserve(scope, callback, timeout_ms);
    result.set(v8::Integer::new(scope, id).into());
}
