use super::media_key_message_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "messageType", get_message_type)
}

fn get_message_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(value) = v8::String::new(s, &v.kind)
    {
        r.set(value.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
