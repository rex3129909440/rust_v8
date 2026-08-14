pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let data = v8::String::new(s, name).ok_or_else(|| "invalid touch handler".to_owned())?;
    crate::webidl::define_accessor_with_data(s, p, name, get, set, data.into())
}
fn get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(s, crate::trace::native_callback_data(s, &a));
    if !super::document::is_document(s, a.this()) {
        return;
    }
    match super::document::handler_value(s, a.this(), &name) {
        Some(v) => r.set(v8::Local::new(s, &v)),
        None => r.set(v8::null(s).into()),
    }
}
fn set(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, _: v8::ReturnValue<'_>) {
    let name = crate::webidl::value_to_string(s, crate::trace::native_callback_data(s, &a));
    let _ = super::document::set_handler(s, a.this(), &name, a.get(0));
}
