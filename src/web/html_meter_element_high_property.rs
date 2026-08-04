use super::html_meter_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "high", get_high, set_high)
}

fn get_high(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| {
        x.high
            .clamp(x.low.clamp(x.min, x.max.max(x.min)), x.max.max(x.min))
    });
}

fn set_high(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_argument(s, a.get(0));
    update(s, a.this(), |x| x.high = value);
}
