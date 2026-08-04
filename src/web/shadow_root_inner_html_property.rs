pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "innerHTML",
        get_inner_html,
        set_inner_html,
    )
}
fn get_inner_html(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if super::shadow_root::record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let html = super::dom_html::serialize_children(s, a.this());
    if let Some(html) = v8::String::new(s, &html) {
        let mut r = r;
        r.set(html.into());
    }
}

fn set_inner_html(
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
