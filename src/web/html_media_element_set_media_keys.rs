use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setMediaKeys", 1, set_media_keys)
}

fn set_media_keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    if scope
        .get_slot::<HtmlMediaElementStore>()
        .and_then(|store| store.records.get(&identity))
        .is_none()
    {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "HTMLMediaElement",
            "setMediaKeys",
            result,
        );
        return;
    }
    let value = arguments.get(0);
    let media_keys =
        (!value.is_null() && !value.is_undefined()).then(|| v8::Global::new(scope, value));
    if let Some(record) = scope
        .get_slot_mut::<HtmlMediaElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.media_keys = media_keys;
        let undefined = v8::undefined(scope);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
            result.set(promise.into());
        }
    }
}
