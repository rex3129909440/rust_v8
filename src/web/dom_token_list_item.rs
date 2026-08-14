pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "item", 1, item)
}
fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = super::dom_token_list::list(scope, arguments.this()) {
        if arguments.length() < 1 {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'item' on 'DOMTokenList': 1 argument required, but only 0 present.",
            );
            return;
        }
        let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
        if let Some(value) = values.get(index).and_then(|v| v8::String::new(scope, v)) {
            result.set(value.into())
        } else {
            result.set(v8::null(scope).into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
