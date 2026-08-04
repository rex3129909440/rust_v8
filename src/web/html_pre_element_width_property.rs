use super::html_pre_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "width", get_width, set_width)
}

fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(width) = scope
        .get_slot::<HtmlPreElementStore>()
        .and_then(|store| {
            store
                .widths
                .get(&arguments.this().get_identity_hash().get())
        })
        .copied()
    {
        result.set(v8::Integer::new(scope, width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = arguments.get(0).int32_value(scope).unwrap_or(0);
    if let Some(current) = scope
        .get_slot_mut::<HtmlPreElementStore>()
        .and_then(|store| {
            store
                .widths
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = width;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
