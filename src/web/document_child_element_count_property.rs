pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "childElementCount", get_child_element_count)
}
fn get_child_element_count(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let count = super::document::document_child_elements(s, a.this()).len() as u32;
    r.set(v8::Integer::new_from_unsigned(s, count).into());
}
