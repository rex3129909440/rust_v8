use super::navigate_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "downloadRequest",
        get_download_request,
    )
}

fn get_download_request(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(download_request) = record.download_request
        && let Some(value) = v8::String::new(scope, &download_request)
    {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}
