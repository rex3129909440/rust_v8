use super::html_video_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "msVideoProcessing",
        get_ms_video_processing,
        set_ms_video_processing,
    )
}

fn get_ms_video_processing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.ms_video_processing) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_ms_video_processing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let valid = matches!(
        value.as_str(),
        "bicubic"
            | "lanczos"
            | "cas"
            | "default"
            | "msSuperResolution"
            | "msGraphicsDriverEnhancement"
    );
    if !valid {
        crate::webidl::throw_type_error(scope, "Invalid video processing type");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.ms_video_processing = value
    });
}
