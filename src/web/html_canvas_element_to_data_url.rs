use super::html_canvas_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "toDataURL", 0, to_data_url)
}

fn to_data_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.width == 0 || snapshot.height == 0 {
        if let Some(value) = v8::String::new(scope, "data:,") {
            result.set(value.into());
        }
        return;
    }
    let Some(bytes) = fingerprinted_png_bytes(
        scope,
        snapshot.width,
        snapshot.height,
        canvas_pixels(scope, &snapshot).as_deref(),
    ) else {
        if let Some(value) = v8::String::new(scope, "data:,") {
            result.set(value.into());
        }
        return;
    };
    let value = format!("data:image/png;base64,{}", encode_base64(&bytes));
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}
