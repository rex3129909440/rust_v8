pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "defaultView", get_default_view)
}
fn get_default_view(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    if let Some(value) = super::document::stored_value(s, a.this(), "defaultView") {
        r.set(v8::Local::new(s, &value));
    } else {
        r.set(v8::null(s).into());
    }
}
