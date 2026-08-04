pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "structuredClone",
        1,
        v8::ConstructorBehavior::Throw,
        structured_clone,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "structuredClone")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.structuredClone".to_owned())
    }
}

fn structured_clone(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'structuredClone' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let context = v8::Global::new(scope, scope.get_entered_or_microtask_context());
    let transfer = match super::structured_clone::transfer_from_options(scope, arguments.get(1)) {
        Ok(transfer) => transfer,
        Err(message) => {
            super::structured_clone::throw_data_clone_error(scope, &message);
            return;
        }
    };
    let context = v8::Local::new(scope, &context);
    match super::structured_clone::clone_into(scope, context, arguments.get(0), transfer) {
        Ok(cloned) => result.set(v8::Local::new(scope, &cloned.value)),
        Err(message) => {
            super::structured_clone::throw_data_clone_error(scope, &message);
        }
    }
}
