pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "doctype", get_doctype)
}
fn get_doctype(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = super::node::children(s, a.this())
        .into_iter()
        .find(|node| super::node::record(s, *node).is_some_and(|record| record.node_type == 10));
    super::document_property_support::return_optional(s, value, r);
}
