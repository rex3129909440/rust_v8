use super::submit_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "submitter", get_submitter)
}

fn get_submitter(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = scope
        .get_slot::<SubmitEventStore>()
        .and_then(|s| s.submitters.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = v {
        r.set(v8::Local::new(scope, &v).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
