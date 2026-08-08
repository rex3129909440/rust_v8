pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "currentScript", get_current_script)
}
fn get_current_script(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if super::document_property_support::ensure(s, a.this()) {
        if let Some(script) = super::document::current_script(s, a.this()) {
            r.set(script.into());
        } else {
            r.set(v8::null(s).into());
        }
    }
}
