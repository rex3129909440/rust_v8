use super::html_video_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "msGetVideoProcessingTypes",
        0,
        ms_get_video_processing_types,
    )
}

fn ms_get_video_processing_types(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let values = v8::Array::new(scope, 6);
    let bicubic = v8::String::new(scope, "bicubic").expect("string");
    let lanczos = v8::String::new(scope, "lanczos").expect("string");
    let cas = v8::String::new(scope, "cas").expect("string");
    let default_value = v8::String::new(scope, "default").expect("string");
    let super_resolution = v8::String::new(scope, "msSuperResolution").expect("string");
    let graphics_driver = v8::String::new(scope, "msGraphicsDriverEnhancement").expect("string");
    let _ = values.set_index(scope, 0, bicubic.into());
    let _ = values.set_index(scope, 1, lanczos.into());
    let _ = values.set_index(scope, 2, cas.into());
    let _ = values.set_index(scope, 3, default_value.into());
    let _ = values.set_index(scope, 4, super_resolution.into());
    let _ = values.set_index(scope, 5, graphics_driver.into());
    result.set(values.into());
}
