use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "click", 0, click)
}

pub(crate) fn click(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.handlers.get("onclick")
        && let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, handler))
    {
        let receiver: v8::Local<v8::Value> = arguments.this().into();
        let event = super::event::create(scope, "click")
            .map(v8::Local::<v8::Value>::from)
            .unwrap_or_else(|_| v8::undefined(scope).into());
        let _ = function.call(scope, receiver, &[event]);
    }
}
