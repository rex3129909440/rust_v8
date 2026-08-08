use super::track_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "track", get_track)
}

fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let track = scope
        .get_slot::<TrackEventStore>()
        .and_then(|store| {
            store
                .tracks
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    let Some(track) = track else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(track) = track {
        result.set(v8::Local::new(scope, &track).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
