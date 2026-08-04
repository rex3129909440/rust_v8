use super::media_stream_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "stream", get_stream)
}

fn get_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let stream = scope
        .get_slot::<MediaStreamEventStore>()
        .and_then(|store| {
            store
                .streams
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    let Some(stream) = stream else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(stream) = stream {
        result.set(v8::Local::new(scope, &stream).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
