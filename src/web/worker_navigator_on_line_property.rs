pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "onLine", get)
}
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::worker_navigator::valid_this(scope, arguments.this()) {
        let value = crate::fingerprint::navigator(scope).on_line;
        result.set(v8::Boolean::new(scope, value).into());
    }
}
