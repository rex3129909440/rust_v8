use super::html_quote_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "cite", get_cite, set_cite)
}

fn get_cite(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = scope
        .get_slot::<HtmlQuoteElementStore>()
        .and_then(|s| s.cite.get(&a.this().get_identity_hash().get()))
    {
        if let Some(v) = v8::String::new(scope, x) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn set_cite(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(x) = scope
        .get_slot_mut::<HtmlQuoteElementStore>()
        .and_then(|s| s.cite.get_mut(&a.this().get_identity_hash().get()))
    {
        *x = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
