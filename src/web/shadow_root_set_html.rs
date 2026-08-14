pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setHTML", 1, set_html)
}
fn set_html(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut value = crate::webidl::value_to_string(scope, a.get(0));
    while let Some(start) = value.to_ascii_lowercase().find("<script") {
        if let Some(end) = value[start..].to_ascii_lowercase().find("</script>") {
            value.replace_range(start..start + end + 9, "");
        } else {
            value.truncate(start);
            break;
        }
    }
    if let Err(message) = super::dom_html::replace_children_with_html(scope, a.this(), &value) {
        crate::webidl::throw_type_error(scope, &message);
    }
}
