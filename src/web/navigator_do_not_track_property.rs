pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "doNotTrack", get)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::navigator::valid_this(scope, arguments.this()) {
        return;
    }
    let value = crate::fingerprint::navigator(scope).do_not_track.clone();
    match value {
        Some(value) => super::navigator::return_string(scope, &value, result),
        None => result.set(v8::null(scope).into()),
    }
}
