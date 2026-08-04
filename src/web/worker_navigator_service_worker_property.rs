pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "serviceWorker", get)
}
fn get(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    if super::worker_navigator::valid_this(s, a.this()) {
        let mut r = r;
        r.set(v8::undefined(s).into());
    }
}
