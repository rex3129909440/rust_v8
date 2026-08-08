pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "innerHTML", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let html = super::dom_html::serialize_children(scope, arguments.this());
    if let Some(html) = v8::String::new(scope, &html) {
        result.set(html.into());
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let html = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Err(message) =
        super::dom_html::replace_children_with_html(scope, arguments.this(), &html)
    {
        crate::webidl::throw_type_error(scope, &message);
    }
}
