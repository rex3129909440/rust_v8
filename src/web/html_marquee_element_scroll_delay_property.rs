use super::html_marquee_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "scrollDelay",
        get_scroll_delay,
        set_scroll_delay,
    )
}

fn get_scroll_delay(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_unsigned(s, a, r, |x| x.scroll_delay);
}

fn set_scroll_delay(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).uint32_value(s).unwrap_or(0);
    update(s, a.this(), |x| x.scroll_delay = v);
}
