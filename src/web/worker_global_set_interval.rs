pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "setInterval",
        1,
        v8::ConstructorBehavior::Throw,
        set_interval,
    )?;
    crate::webidl::define_global(scope, "setInterval", function.into())
}
pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setInterval", 1, set_interval)
}
fn set_interval(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::set_interval(s, a, r);
}
