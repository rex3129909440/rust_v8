use super::html_media_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setSinkId", 1, set_sink_id)
}

fn set_sink_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "HTMLMediaElement",
            "setSinkId",
            result,
        );
        return;
    }
    let sink_id = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.sink_id = sink_id);
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}
