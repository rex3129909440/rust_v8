pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "setInterval",
        1,
        v8::ConstructorBehavior::Throw,
        set_interval,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "setInterval")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.setInterval".to_owned())
    }
}

fn set_interval(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'setInterval' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let callback = arguments.get(0);
    let delay = arguments.get(1).number_value(scope).unwrap_or(0.0);
    let mut callback_arguments = Vec::new();
    for index in 2..arguments.length() {
        callback_arguments.push(v8::Global::new(scope, arguments.get(index)));
    }
    let id = super::timer_state::reserve_interval(scope, callback, callback_arguments, delay);
    result.set(v8::Integer::new(scope, id).into());
}
