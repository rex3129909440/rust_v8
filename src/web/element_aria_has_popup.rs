pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "ariaHasPopup", get_aria_has_popup, set_aria_has_popup)
}
fn get_aria_has_popup(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::get_string(s, a, r, "aria-haspopup")
}
fn set_aria_has_popup(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::set_string(s, a, "aria-haspopup")
}
