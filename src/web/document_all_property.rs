pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "all", get_all)
}
fn get_all(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    if super::document_property_support::return_stored(s, a.this(), "all", r) {
        return;
    }
    match super::html_all_collection::create_for_document(s, a.this()) {
        Ok(value) => {
            super::document::remember_value(s, a.this(), "all", value.into());
            r.set(value.into());
        }
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
