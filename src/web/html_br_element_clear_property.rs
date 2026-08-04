use super::html_br_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "clear", get_clear, set_clear)
}

fn get_clear(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = s
        .get_slot::<HtmlBrElementStore>()
        .and_then(|q| q.clear.get(&a.this().get_identity_hash().get()))
    {
        if let Some(v) = v8::String::new(s, x) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_clear(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if let Some(x) = s
        .get_slot_mut::<HtmlBrElementStore>()
        .and_then(|q| q.clear.get_mut(&a.this().get_identity_hash().get()))
    {
        *x = v
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
