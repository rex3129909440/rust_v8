pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "setTimeout",
        1,
        v8::ConstructorBehavior::Throw,
        set_timeout,
    )?;
    crate::webidl::define_global(scope, "setTimeout", function.into())
}
pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setTimeout", 1, set_timeout)
}
fn set_timeout(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::set_timeout(s, a, r);
}
