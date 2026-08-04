use super::document_picture_in_picture_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "window", get_window)
}

fn get_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let window = scope
        .get_slot::<DocumentPictureInPictureEventStore>()
        .and_then(|store| {
            store
                .windows
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(window) = window {
        result.set(v8::Local::new(scope, &window).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
