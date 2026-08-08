use super::html_form_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "reset", 0, reset)
}

fn reset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event = super::event_target::create_event(scope, "reset");
    if !super::event_target::dispatch(scope, arguments.this(), event) {
        return;
    }
    for control in collect_controls(scope, arguments.this()) {
        reset_control(scope, control);
    }
    update(scope, arguments.this(), |record| record.reset_count += 1);
}
