pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "queueMicrotask",
        1,
        v8::ConstructorBehavior::Throw,
        queue_microtask,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "queueMicrotask")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.queueMicrotask".to_owned())
    }
}

fn queue_microtask(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'queueMicrotask' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'queueMicrotask' on 'Window': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    scope.enqueue_microtask(callback);
}
