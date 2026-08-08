pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "scrollingElement", get_scrolling_element)
}
fn get_scrolling_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = super::document::document_child_elements(s, a.this())
        .into_iter()
        .next();
    super::document_property_support::return_optional(s, value, r);
}
