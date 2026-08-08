pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "appCodeName", get)
}
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::worker_navigator::valid_this(scope, arguments.this()) {
        return;
    }
    let value = crate::fingerprint::navigator(scope).app_code_name.clone();
    super::worker_navigator::return_string(scope, &mut result, &value);
}
