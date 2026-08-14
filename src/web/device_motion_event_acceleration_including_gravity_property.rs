use super::device_motion_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "accelerationIncludingGravity",
        get_acceleration_including_gravity,
    )
}

fn get_acceleration_including_gravity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        match x.acceleration_including_gravity {
            Some(value) => r.set(v8::Local::new(s, &value).into()),
            None => r.set(v8::null(s).into()),
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
