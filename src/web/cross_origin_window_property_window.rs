pub(crate) fn create_getter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    create(scope, iframe_id, "get window", get_window)
}

pub(crate) fn create_ancestor_getter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    create(scope, iframe_id, "get window", get_ancestor_window)
}

fn get_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = super::html_i_frame_element::cross_origin_property_value_for_iframe(
        scope,
        crate::trace::native_callback_data(scope, &arguments)
            .int32_value(scope)
            .unwrap_or_default(),
        "window",
    ) {
        result.set(value);
    }
}

fn get_ancestor_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) =
        super::html_i_frame_element::cross_origin_ancestor_property_value_for_iframe(
            scope,
            crate::trace::native_callback_data(scope, &arguments)
                .int32_value(scope)
                .unwrap_or_default(),
            "window",
        )
    {
        result.set(value);
    }
}

fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let data = v8::Integer::new(scope, iframe_id);
    crate::webidl::create_function_with_data(
        scope,
        name,
        0,
        v8::ConstructorBehavior::Throw,
        callback,
        data.into(),
    )
}
