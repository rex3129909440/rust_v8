use super::rtc_peer_connection_ice_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "candidate", get_candidate)
}

fn get_candidate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<RtcPeerConnectionIceEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    match value {
        Some(Some(candidate)) => result.set(v8::Local::new(scope, &candidate).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
