pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "whatToShow", get_what_to_show)
}
fn get_what_to_show(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = super::node_iterator::record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.what_to_show as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
