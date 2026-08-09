pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "timeline", get_timeline)
}
fn get_timeline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match value(s, a.this()) {
        Ok(value) => r.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}

pub(crate) fn value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if !super::document::is_document(scope, document) {
        return Err("Illegal invocation".to_owned());
    }
    if let Some(stored) = super::document::stored_value(scope, document, "timeline") {
        return v8::Local::<v8::Object>::try_from(v8::Local::new(scope, stored))
            .map_err(|_| "document timeline is not an object".to_owned());
    }
    let timeline = super::document_timeline::create(scope, 0.0)?;
    super::document::remember_value(scope, document, "timeline", timeline.into());
    Ok(timeline)
}
