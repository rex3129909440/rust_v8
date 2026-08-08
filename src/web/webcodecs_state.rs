pub(crate) fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "InvalidStateError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

pub(crate) fn reject_invalid_state(
    scope: &mut v8::PinScope<'_, '_>,
    message: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "InvalidStateError".to_owned())
    else {
        return;
    };
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn throw_argument_type(
    scope: &mut v8::PinScope<'_, '_>,
    operation: &str,
    interface: &str,
) {
    crate::webidl::throw_type_error(
        scope,
        &format!("Failed to execute '{operation}': parameter 1 is not of type '{interface}'."),
    );
}

pub(crate) fn encoding_error<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    message: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    super::dom_exception::create(scope, message.to_owned(), "EncodingError".to_owned()).ok()
}

pub(crate) fn fire_dequeue(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
) {
    let Ok(event) = super::event::create(scope, "dequeue") else {
        return;
    };
    let _ = super::event_target::dispatch(scope, target, event);
    if let Some(handler) = handler
        && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
}
