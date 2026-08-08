pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "activeViewTransition",
        get_active_view_transition,
    )
}

fn get_active_view_transition(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::element::cached_reflected_value(scope, arguments.this(), "activeViewTransition") {
        Some(value) => result.set(v8::Local::new(scope, &value)),
        None => result.set(v8::null(scope).into()),
    }
}
