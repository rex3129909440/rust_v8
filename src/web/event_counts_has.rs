use super::event_counts::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "has", 1, has)
}

fn has(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(s, a.get(0));
    if let Some(values) = snapshot(s, a.this()) {
        r.set(v8::Boolean::new(s, values.iter().any(|(key, _)| key == &name)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
