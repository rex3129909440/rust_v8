pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "clientTop", get_client_top)
}

fn get_client_top(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_some() {
        let layout = super::element_layout::compute(scope, arguments.this());
        result
            .set(v8::Integer::new(scope, super::element_layout::rounded(layout.border_top)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
