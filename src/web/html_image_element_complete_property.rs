use super::html_image_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "complete", get_complete)
}

fn get_complete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(
            v8::Boolean::new(scope, record.request_state != ImageRequestState::Pending).into(),
        );
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
