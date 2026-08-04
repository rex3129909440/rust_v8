pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "ariaNotify", 1, aria_notify)
}

fn aria_notify(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let notification = crate::webidl::value_to_string(scope, arguments.get(0));
    super::document::set_string_value(
        scope,
        arguments.this(),
        "lastAriaNotification",
        &notification,
    );
}
