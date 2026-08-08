pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "cancelAnimationFrame",
        1,
        v8::ConstructorBehavior::Throw,
        cancel_animation_frame,
    )?;
    crate::webidl::define_global(scope, "cancelAnimationFrame", function.into())
}
pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        object,
        "cancelAnimationFrame",
        1,
        cancel_animation_frame,
    )
}
fn cancel_animation_frame(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::worker_global_scope::cancel_animation_frame(s, a, r);
}
