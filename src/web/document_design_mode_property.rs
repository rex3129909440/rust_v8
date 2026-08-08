pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "designMode", get_design_mode, set_design_mode)
}
fn get_design_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::document_property_support::get_string(s, a, r, "designMode", "off")
}
fn set_design_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0)).to_ascii_lowercase();
    if value == "on" || value == "off" {
        super::document::set_string_value(s, a.this(), "designMode", &value);
    }
}
