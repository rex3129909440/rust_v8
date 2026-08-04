use super::drag_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "dataTransfer", get_data_transfer)
}

fn get_data_transfer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match s
        .get_slot::<DragEventStore>()
        .and_then(|x| x.data_transfers.get(&a.this().get_identity_hash().get()))
    {
        Some(Some(value)) => r.set(v8::Local::new(s, value).into()),
        Some(None) => r.set(v8::null(s).into()),
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
