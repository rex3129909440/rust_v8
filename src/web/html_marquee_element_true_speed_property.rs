use super::html_marquee_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "trueSpeed",
        get_true_speed,
        set_true_speed,
    )
}

fn get_true_speed(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, record.true_speed).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_true_speed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(s);
    update(s, a.this(), |x| x.true_speed = v);
}
