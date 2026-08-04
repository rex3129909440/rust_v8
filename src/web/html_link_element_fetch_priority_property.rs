use super::html_link_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "fetchPriority",
        get_fetch_priority,
        set_fetch_priority,
    )
}

fn get_fetch_priority(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.fetch_priority);
}

fn set_fetch_priority(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    let value = if matches!(value.as_str(), "high" | "low" | "auto") {
        value
    } else {
        "auto".to_owned()
    };
    update(s, a.this(), |x| x.fetch_priority = value);
}
