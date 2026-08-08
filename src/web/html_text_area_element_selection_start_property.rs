use super::html_text_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "selectionStart",
        get_selection_start,
        set_selection_start,
    )
}

fn get_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.selection_start).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_selection_start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let requested = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        let value = requested.min(text_len(&record.value));
        record.selection_start = value;
        if value > record.selection_end {
            record.selection_end = value;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
