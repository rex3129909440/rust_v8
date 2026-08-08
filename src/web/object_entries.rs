pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    crate::webidl::replace_intrinsic_method(scope, "Object", "entries", 1, entries)
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0))
        && let Some(entries) =
            super::html_i_frame_element::cross_origin_window_index_entries(scope, object)
    {
        result.set(entries.into());
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
