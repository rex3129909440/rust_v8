use super::error_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "lineno", get_lineno)
}

fn get_lineno(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, record.lineno).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
