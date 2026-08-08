use super::html_html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "version", get_version, set_version)
}

fn get_version(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<HtmlHtmlElementStore>()
        .and_then(|store| store.versions.get(&a.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, value) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_version(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(current) = scope
        .get_slot_mut::<HtmlHtmlElementStore>()
        .and_then(|store| store.versions.get_mut(&a.this().get_identity_hash().get()))
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
