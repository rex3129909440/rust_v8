use super::text_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "data", get_data)
}

fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(data) = scope
        .get_slot::<TextEventStore>()
        .and_then(|s| s.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(scope, &data) {
        r.set(v.into())
    }
}
