pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "hasChildNodes", 0, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::node::record(scope, arguments.this()) {
        Some(record) => {
            result.set(v8::Boolean::new(scope, !record.children.is_empty()).into());
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
