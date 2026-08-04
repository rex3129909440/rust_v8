use super::html_legend_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "align", get_align, set_align)
}

fn get_align(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(align) = scope
        .get_slot::<HtmlLegendElementStore>()
        .and_then(|store| store.align.get(&a.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, align) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_align(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(align) = scope
        .get_slot_mut::<HtmlLegendElementStore>()
        .and_then(|store| store.align.get_mut(&a.this().get_identity_hash().get()))
    {
        *align = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
