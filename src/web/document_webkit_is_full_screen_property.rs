pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "webkitIsFullScreen", get_webkit_is_full_screen)
}
fn get_webkit_is_full_screen(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if super::document_property_support::ensure(s, a.this()) {
        let active = super::document::stored_value(s, a.this(), "fullscreenElement").is_some();
        r.set(v8::Boolean::new(s, active).into());
    }
}
