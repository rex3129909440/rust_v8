pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "languages", get)
}
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::worker_navigator::valid_this(scope, arguments.this()) {
        return;
    }
    if let Some(languages) = super::worker_navigator::languages_object(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &languages).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
