use super::html_geolocation_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onvalidationstatuschange",
        get_on_validation_status_change,
        set_on_validation_status_change,
    )
}

fn get_on_validation_status_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.on_validation_status_change)
}

fn set_on_validation_status_change(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.on_validation_status_change = v)
}
