pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        s,
        p,
        "xmlStandalone",
        get_xml_standalone,
        set_xml_standalone,
    )
}
fn get_xml_standalone(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    if super::document_property_support::return_stored(s, a.this(), "xmlStandalone", r) {
        return;
    }
    r.set(v8::Boolean::new(s, false).into());
}
fn set_xml_standalone(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = v8::Boolean::new(s, a.get(0).boolean_value(s));
    super::document::remember_value(s, a.this(), "xmlStandalone", value.into());
}
