use super::offline_audio_completion_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "renderedBuffer", get_rendered_buffer)
}

fn get_rendered_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(buffer) = scope
        .get_slot::<OfflineAudioCompletionEventStore>()
        .and_then(|store| {
            store
                .rendered_buffers
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, buffer).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
