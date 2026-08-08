use super::sensor_error_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "error", error)
}

fn error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot::<SensorErrorEventStore>()
        .and_then(|x| x.errors.get(&a.this().get_identity_hash().get()))
        .cloned()
    {
        r.set(v8::Local::new(s, &v).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
