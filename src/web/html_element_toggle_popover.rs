use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "togglePopover", 0, toggle_popover)
}

pub(crate) fn toggle_popover(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let force = arguments.get(0);
    let forced_value = (!force.is_undefined()).then(|| force.boolean_value(scope));
    let identity = arguments.this().get_identity_hash().get();
    let visible = if let Some(record) = scope
        .get_slot_mut::<HtmlElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.popover_visible = forced_value.unwrap_or(!record.popover_visible);
        Some(record.popover_visible)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        None
    };
    if let Some(visible) = visible {
        result.set(v8::Boolean::new(scope, visible).into());
    }
}
