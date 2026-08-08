use super::html_form_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "requestSubmit", 0, request_submit)
}

fn request_submit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !current.no_validate && !controls_valid(scope, arguments.this()) {
        return;
    }
    let event = super::event_target::create_event(scope, "submit");
    if super::event_target::dispatch(scope, arguments.this(), event) {
        update(scope, arguments.this(), |record| record.submit_count += 1);
    }
}
