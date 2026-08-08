pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "queueMicrotask",
        1,
        v8::ConstructorBehavior::Throw,
        queue_microtask,
    )?;
    crate::webidl::define_global(scope, "queueMicrotask", function.into())
}
pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "queueMicrotask", 1, queue_microtask)
}
fn queue_microtask(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::queue_microtask(s, a, r);
}
