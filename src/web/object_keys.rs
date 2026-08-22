pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    crate::webidl::replace_intrinsic_method(scope, "Object", "keys", 1, keys)
}

fn keys<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0))
        && let Some(keys) =
            super::html_i_frame_element::cross_origin_window_index_keys(scope, object)
    {
        result.set(keys.into());
        return;
    }
    let Ok(original) =
        v8::Local::<v8::Function>::try_from(crate::trace::native_callback_data(scope, &arguments))
    else {
        return;
    };
    if let Some(value) = original.call(scope, arguments.this().into(), &[arguments.get(0)]) {
        if let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0))
            && let Ok(keys) = v8::Local::<v8::Array>::try_from(value)
            && let Some(keys) =
                crate::browser_surface::virtualize_webview_window_keys(scope, object, keys, false)
        {
            result.set(keys.into());
        } else {
            result.set(value);
        }
    }
}
