pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "plugins", get_plugins)
}
fn get_plugins(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if super::document_property_support::ensure(s, a.this()) {
        super::document_property_support::legacy_collection(s, a.this(), "plugins", r)
    }
}
