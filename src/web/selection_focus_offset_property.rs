pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "focusOffset", get_focus_offset)
}
fn get_focus_offset(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = super::selection::record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, v.focus_offset).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
