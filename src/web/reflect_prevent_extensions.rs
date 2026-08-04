pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    crate::webidl::replace_intrinsic_method(
        scope,
        "Reflect",
        "preventExtensions",
        1,
        prevent_extensions,
    )
}

fn prevent_extensions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0))
        && super::html_i_frame_element::is_cross_origin_window_proxy(scope, object)
    {
        super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        return;
    }
    let Ok(original) =
        v8::Local::<v8::Function>::try_from(crate::trace::native_callback_data(scope, &arguments))
    else {
        return;
    };
    if let Some(value) = original.call(scope, arguments.this().into(), &[arguments.get(0)]) {
        result.set(value);
    }
}
