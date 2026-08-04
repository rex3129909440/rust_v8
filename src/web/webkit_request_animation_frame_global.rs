pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "webkitRequestAnimationFrame",
        1,
        v8::ConstructorBehavior::Throw,
        webkit_request_animation_frame,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "webkitRequestAnimationFrame")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.webkitRequestAnimationFrame".to_owned())
    }
}

fn webkit_request_animation_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'webkitRequestAnimationFrame' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'webkitRequestAnimationFrame' on 'Window': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let id = super::animation_frame_state::reserve(scope, callback);
    result.set(v8::Integer::new(scope, id).into());
}
