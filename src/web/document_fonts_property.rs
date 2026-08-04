pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "fonts", get_fonts)
}
fn get_fonts(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    if super::document_property_support::return_stored(s, a.this(), "fonts", r) {
        return;
    }
    match super::font_face_set::create(s) {
        Ok(value) => {
            super::document::remember_value(s, a.this(), "fonts", value.into());
            r.set(value.into());
        }
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
