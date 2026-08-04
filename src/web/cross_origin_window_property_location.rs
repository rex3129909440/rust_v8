pub(crate) fn create_getter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let data = v8::Integer::new(scope, iframe_id);
    crate::webidl::create_function_with_data(
        scope,
        "get location",
        0,
        v8::ConstructorBehavior::Throw,
        get_location,
        data.into(),
    )
}

pub(crate) fn create_setter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let data = v8::Integer::new(scope, iframe_id);
    crate::webidl::create_function_with_data(
        scope,
        "set location",
        1,
        v8::ConstructorBehavior::Throw,
        set_location,
        data.into(),
    )
}

pub(crate) fn create_ancestor_getter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let data = v8::Integer::new(scope, iframe_id);
    crate::webidl::create_function_with_data(
        scope,
        "get location",
        0,
        v8::ConstructorBehavior::Throw,
        get_ancestor_location,
        data.into(),
    )
}

pub(crate) fn create_ancestor_setter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let data = v8::Integer::new(scope, iframe_id);
    crate::webidl::create_function_with_data(
        scope,
        "set location",
        1,
        v8::ConstructorBehavior::Throw,
        set_ancestor_location,
        data.into(),
    )
}

fn get_location(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = super::html_i_frame_element::cross_origin_property_value_for_iframe(
        scope,
        crate::trace::native_callback_data(scope, &arguments)
            .int32_value(scope)
            .unwrap_or_default(),
        "location",
    ) {
        result.set(value);
    }
}

fn set_location(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let iframe_id = crate::trace::native_callback_data(scope, &arguments)
        .int32_value(scope)
        .unwrap_or_default();
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Err(message) =
        super::html_i_frame_element::navigate_cross_origin_location(scope, iframe_id, value)
    {
        crate::webidl::throw_type_error(scope, &message);
    }
}

fn get_ancestor_location(
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
            "location",
        )
    {
        result.set(value);
    }
}

fn set_ancestor_location(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::cross_origin_ancestor_location::navigate(scope, value);
}
