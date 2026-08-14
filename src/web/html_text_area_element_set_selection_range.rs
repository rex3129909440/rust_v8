use super::html_text_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "setSelectionRange",
        2,
        set_selection_range,
    )
}

fn set_selection_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let start = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let end = arguments.get(1).uint32_value(scope).unwrap_or(0);
    let direction = if arguments.get(2).is_undefined() {
        "none".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(2))
    };
    let effective_length = record(scope, arguments.this()).map(|record| {
        if record.value_dirty {
            text_len(&record.value)
        } else {
            text_len(&super::node::node_text(scope, arguments.this()))
        }
    });
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        let length = effective_length.unwrap_or(0);
        record.selection_end = end.min(length);
        record.selection_start = start.min(record.selection_end);
        record.selection_direction = if direction == "forward" || direction == "backward" {
            direction
        } else {
            "none".to_owned()
        };
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
