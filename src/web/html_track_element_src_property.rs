use super::html_track_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "src", get_src, set_src)
}

fn get_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_getter(s, a, r, |x| &x.src);
}

fn set_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    string_setter(s, a, |x, v| {
        x.src = v;
        x.ready_state = if x.src.is_empty() { NONE } else { LOADING };
    });
}
