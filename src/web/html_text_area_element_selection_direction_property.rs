use super::html_text_area_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "selectionDirection",
        get_selection_direction,
        set_selection_direction,
    )
}

fn get_selection_direction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |x| &x.selection_direction);
}

fn set_selection_direction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if value == "forward" || value == "backward" || value == "none" {
        value
    } else {
        "none".to_owned()
    };
    if let Some(record) = scope
        .get_slot_mut::<HtmlTextAreaElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.selection_direction = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
