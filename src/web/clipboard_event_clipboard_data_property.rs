use super::clipboard_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "clipboardData", get_clipboard_data)
}

fn get_clipboard_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<ClipboardEventStore>()
        .and_then(|store| store.data.get(&arguments.this().get_identity_hash().get()))
        .cloned();
    match value {
        Some(Some(value)) => result.set(v8::Local::new(scope, &value).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
