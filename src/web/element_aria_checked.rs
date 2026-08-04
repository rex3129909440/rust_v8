pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "ariaChecked", get_aria_checked, set_aria_checked)
}
fn get_aria_checked(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::get_string(s, a, r, "aria-checked")
}
fn set_aria_checked(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::set_string(s, a, "aria-checked")
}
