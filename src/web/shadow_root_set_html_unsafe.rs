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
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'setHTMLUnsafe' on 'ShadowRoot': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(value) = crate::webidl::dom_string_with_context(
        scope,
        a.get(0),
        "Failed to execute 'setHTMLUnsafe' on 'ShadowRoot'",
    ) else {
        return;
    };
    if let Err(message) = super::dom_html::replace_children_with_html(scope, a.this(), &value) {
        crate::webidl::throw_type_error(scope, &message);
    }
}
