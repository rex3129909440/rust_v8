pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "domain", get_domain, set_domain)
}
fn get_domain(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::document_property_support::get_string(s, a, r, "domain", &crate::page_init::host(s))
}
fn set_domain(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0)).to_ascii_lowercase();
    let host = crate::page_init::host(s);
    if value.is_empty() && host.is_empty() || value != host && !host.ends_with(&format!(".{value}"))
    {
        super::node::throw_dom_exception(
            s,
            "SecurityError",
            "The document domain cannot be changed to this value",
        );
        return;
    }
    super::document::set_string_value(s, a.this(), "domain", &value);
}
