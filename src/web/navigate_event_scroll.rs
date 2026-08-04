use super::navigate_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "scroll", 0, scroll)
}

fn scroll(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.trusted_navigation {
        throw_dom_exception(
            scope,
            "SecurityError",
            "scroll() may only be called on a trusted navigate event",
        );
    } else if !record.intercepted {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "The navigation has not been intercepted",
        );
    }
}
