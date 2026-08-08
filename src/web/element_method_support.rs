pub(crate) fn ensure(scope: &mut v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    if super::element::record(scope, element).is_some() {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn owner_document<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::node::record(scope, element)
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document))
}

pub(crate) fn scroll_coordinates(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> (f64, f64) {
    if let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        let left = v8::String::new(scope, "left")
            .and_then(|key| options.get(scope, key.into()))
            .and_then(|value| value.number_value(scope))
            .unwrap_or(0.0);
        let top = v8::String::new(scope, "top")
            .and_then(|key| options.get(scope, key.into()))
            .and_then(|value| value.number_value(scope))
            .unwrap_or(0.0);
        (left, top)
    } else {
        (
            arguments.get(0).number_value(scope).unwrap_or(0.0),
            arguments.get(1).number_value(scope).unwrap_or(0.0),
        )
    }
}

pub(crate) fn resolved_undefined<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let undefined = v8::undefined(scope);
    super::writable_stream::resolved_promise(scope, undefined.into())
}
