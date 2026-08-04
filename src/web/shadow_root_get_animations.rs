pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getAnimations", 0, get_animations)
}
fn get_animations(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(scope, a.this()).is_some() {
        r.set(v8::Array::new(scope, 0).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
