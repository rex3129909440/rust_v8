pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "ariaRowSpan", get_aria_row_span, set_aria_row_span)
}
fn get_aria_row_span(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::get_string(s, a, r, "aria-rowspan")
}
fn set_aria_row_span(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    super::element_aria_reflection::set_string(s, a, "aria-rowspan")
}
