use super::html_select_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "showPicker", 0, show_picker)
}

fn show_picker(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(x) = scope
        .get_slot_mut::<HtmlSelectElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.picker_open = true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
