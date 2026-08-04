pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setHTMLUnsafe", 1, set_html_unsafe)
}
fn set_html_unsafe(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Err(message) = super::dom_html::replace_children_with_html(scope, a.this(), &value) {
        crate::webidl::throw_type_error(scope, &message);
    }
}
