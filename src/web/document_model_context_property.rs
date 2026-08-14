pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "modelContext", get)
}
fn get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(s, a.this()) {
        return;
    }
    match super::document::model_context(s, a.this()) {
        Some(v) => r.set(v8::Local::new(s, &v).into()),
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
