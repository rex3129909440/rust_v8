use super::html_table_caption_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "align", get_align, set_align)
}

fn get_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<HtmlTableCaptionElementStore>()
        .and_then(|store| store.align.get(&arguments.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(current) = scope
        .get_slot_mut::<HtmlTableCaptionElementStore>()
        .and_then(|store| {
            store
                .align
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
