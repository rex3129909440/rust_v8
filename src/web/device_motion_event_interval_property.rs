use super::device_motion_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "interval", get_interval)
}

fn get_interval(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.interval).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
