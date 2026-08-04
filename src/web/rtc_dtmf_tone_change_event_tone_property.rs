use super::rtc_dtmf_tone_change_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "tone", get_tone)
}

fn get_tone(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(tone) = scope
        .get_slot::<RtcDtmfToneChangeEventStore>()
        .and_then(|store| store.tones.get(&arguments.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, tone) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
