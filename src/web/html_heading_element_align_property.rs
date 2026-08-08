use super::html_heading_element::*;

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
    if let Some(value) = scope
        .get_slot::<HtmlHeadingElementStore>()
        .and_then(|store| store.alignments.get(&a.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, value) {
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
    if let Some(current) = scope
        .get_slot_mut::<HtmlHeadingElementStore>()
        .and_then(|store| {
            store
                .alignments
                .get_mut(&a.this().get_identity_hash().get())
        })
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
