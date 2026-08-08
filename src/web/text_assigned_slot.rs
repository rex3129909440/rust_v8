pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "assignedSlot", get_assigned_slot)
}

fn get_assigned_slot(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::text::data_if_text(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::html_slot_element::assigned_slot(scope, arguments.this()) {
        Some(slot) => result.set(slot.into()),
        None => result.set(v8::null(scope).into()),
    }
}
