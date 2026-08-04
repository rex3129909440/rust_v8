use super::html_video_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "requestVideoFrameCallback",
        1,
        request_video_frame_callback,
    )
}

fn request_video_frame_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Callback must be callable");
        return;
    };
    let callback = v8::Global::new(scope, callback);
    let identity = arguments.this().get_identity_hash().get();
    let id = if let Some(record) = scope
        .get_slot_mut::<HtmlVideoElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        let id = record.next_callback_id;
        record.next_callback_id = record.next_callback_id.wrapping_add(1).max(1);
        record.callbacks.insert(id, callback);
        Some(id)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        None
    };
    if let Some(id) = id {
        result.set(v8::Integer::new_from_unsigned(scope, id).into());
    }
}
