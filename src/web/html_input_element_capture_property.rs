use super::html_input_element::*;
pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "capture", get, set)
}
fn get(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    get_reflected_string(s, a, r, "capture")
}
fn set(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, _: v8::ReturnValue<'_>) {
    set_reflected_string(s, a, "capture")
}
