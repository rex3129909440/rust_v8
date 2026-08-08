use super::event_counts::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)
}

fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(values) = snapshot(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, values.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
