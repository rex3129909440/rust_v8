pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "host", get_host)
}
fn get_host(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = super::shadow_root::record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.host).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
