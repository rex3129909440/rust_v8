pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    crate::webidl::replace_intrinsic_method(
        scope,
        "Reflect",
        "getOwnPropertyDescriptor",
        2,
        get_own_property_descriptor,
    )
}

fn get_own_property_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0))
        && super::html_i_frame_element::is_cross_origin_window_proxy(scope, object)
        && is_cross_origin_symbol(scope, arguments.get(1))
    {
        let descriptor = super::cross_origin_window_descriptors::data_descriptor(
            scope,
            v8::undefined(scope).into(),
            false,
            false,
            true,
        );
        result.set(descriptor.into());
        return;
    }
    let Ok(original) =
        v8::Local::<v8::Function>::try_from(crate::trace::native_callback_data(scope, &arguments))
    else {
        return;
    };
    if let Some(value) = original.call(
        scope,
        arguments.this().into(),
        &[arguments.get(0), arguments.get(1)],
    ) {
        result.set(value);
    }
}

fn is_cross_origin_symbol(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> bool {
    value.strict_equals(v8::Symbol::get_to_string_tag(scope).into())
        || value.strict_equals(v8::Symbol::get_has_instance(scope).into())
        || value.strict_equals(v8::Symbol::get_is_concat_spreadable(scope).into())
}
