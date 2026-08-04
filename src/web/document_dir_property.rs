pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "dir", get_dir, set_dir)
}
fn get_dir(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = super::document::document_child_elements(s, a.this())
        .into_iter()
        .next()
        .and_then(|root| super::element::attribute_value(s, root, "dir"))
        .unwrap_or_default();
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into());
    }
}
fn set_dir(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    if let Some(root) = super::document::document_child_elements(s, a.this())
        .into_iter()
        .next()
    {
        super::element::set_attribute_value(s, root, "dir".to_owned(), value);
    }
}
