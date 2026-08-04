use super::html_dialog_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "requestClose", 0, request_close)
}

fn request_close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if !record(scope, arguments.this()).is_some_and(|record| record.open) {
        return;
    }
    let cancel_event = super::event_target::create_event(scope, "cancel");
    if super::event_target::dispatch(scope, arguments.this(), cancel_event) {
        close(scope, arguments, result);
    }
}
