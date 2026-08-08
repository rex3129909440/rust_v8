use super::html_geolocation_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "autolocate",
        get_autolocate,
        set_autolocate,
    )
}

fn get_autolocate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |x| x.autolocate)
}

fn set_autolocate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(s);
    update(s, a.this(), |x| x.autolocate = v)
}
