pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get origin",
        0,
        v8::ConstructorBehavior::Throw,
        get_origin,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set origin",
        1,
        v8::ConstructorBehavior::Throw,
        set_origin,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "origin")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.origin".to_owned())
    }
}

fn get_origin(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let window = scope.get_current_context().global(scope);
    let origin = super::html_i_frame_element::origin_for_window(scope, window);
    if let Some(origin) = v8::String::new(scope, &origin) {
        result.set(origin.into());
    }
}

fn set_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let global = scope.get_current_context().global(scope);
    let Some(key) = v8::String::new(scope, "origin") else {
        return;
    };
    let _ = global.define_own_property(
        scope,
        key.into(),
        arguments.get(0),
        v8::PropertyAttribute::NONE,
    );
}
