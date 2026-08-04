pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getBoundingClientRect",
        0,
        get_bounding_client_rect,
    )
}

fn get_bounding_client_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let rect = super::element_layout::compute(scope, arguments.this()).rect();
    match super::dom_rect::create(scope, rect) {
        Ok(rect) => result.set(rect.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
