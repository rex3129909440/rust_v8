use super::mouse_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "fromElement", get_from_element)
}

fn get_from_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let mut r = r;
    if matches!(record.event_type.as_str(), "mouseover" | "mouseenter")
        && let Some(value) = record.related_target
    {
        r.set(v8::Local::new(s, &value));
    } else {
        r.set(v8::null(s).into());
    }
}
