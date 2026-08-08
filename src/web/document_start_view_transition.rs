pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "startViewTransition",
        0,
        start_view_transition,
    )
}

fn start_view_transition(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    match super::view_transition::create(scope, Vec::new(), Some(arguments.this().into())) {
        Ok(transition) => {
            super::document::set_object_value(
                scope,
                arguments.this(),
                "activeViewTransition",
                transition,
            );
            result.set(transition.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
