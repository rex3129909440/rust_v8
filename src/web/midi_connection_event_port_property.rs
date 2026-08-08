use super::midi_connection_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "port", get_port)
}

fn get_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match scope
        .get_slot::<MidiConnectionEventStore>()
        .and_then(|store| store.ports.get(&arguments.this().get_identity_hash().get()))
        .cloned()
    {
        Some(Some(value)) => result.set(v8::Local::new(scope, &value).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
