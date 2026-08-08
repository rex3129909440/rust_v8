pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "xmlVersion", get_xml_version, set_xml_version)
}
fn get_xml_version(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::document_property_support::get_string(s, a, r, "xmlVersion", "1.0")
}
fn set_xml_version(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if value != "1.0" && value != "1.1" {
        super::node::throw_dom_exception(
            s,
            "NotSupportedError",
            "Only XML versions 1.0 and 1.1 are supported",
        );
    } else {
        super::document::set_string_value(s, a.this(), "xmlVersion", &value);
    }
}
