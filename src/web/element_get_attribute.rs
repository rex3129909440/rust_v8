pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getAttribute", 1, call)
}

fn call(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    match super::element::attribute_value(scope, arguments.this(), &name) {
        Some(value) => {
            if let Some(value) = v8::String::new(scope, &value) {
                result.set(value.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}
