use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onwaitingforkey",
        get_on_waiting_for_key,
        set_on_waiting_for_key,
    )
}

fn get_on_waiting_for_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.on_waiting_for_key.clone());
}

fn set_on_waiting_for_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(s, a.get(0));
    update(s, a.this(), |record| record.on_waiting_for_key = value);
}
