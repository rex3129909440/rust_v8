pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "webkitVisibilityState", get_state)
}
fn get_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if super::document_property_support::ensure(s, a.this()) {
        let state = crate::fingerprint::edge(s)
            .document
            .visibility_state
            .as_deref()
            .unwrap_or("visible");
        if let Some(value) = v8::String::new(s, state) {
            r.set(value.into());
        }
    }
}
