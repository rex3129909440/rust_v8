pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "contentType", get_content_type)
}
fn get_content_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(value) = super::document::content_type_value(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into());
    }
}
