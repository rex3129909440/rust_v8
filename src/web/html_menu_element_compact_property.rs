use super::html_menu_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "compact", get_compact, set_compact)
}

fn get_compact(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<HtmlMenuElementStore>()
        .and_then(|store| store.compact.get(&a.this().get_identity_hash().get()))
        .copied()
    {
        r.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_compact(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    if let Some(current) = scope
        .get_slot_mut::<HtmlMenuElementStore>()
        .and_then(|store| store.compact.get_mut(&a.this().get_identity_hash().get()))
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
