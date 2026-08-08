use super::text_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "initTextEvent", 1, init_text_event)
}

fn init_text_event(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<TextEventStore>()
        .is_some_and(|s| s.records.contains_key(&a.this().get_identity_hash().get()))
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let data = crate::webidl::value_to_string(scope, a.get(4));
    if let Some(v) = scope
        .get_slot_mut::<TextEventStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        *v = data;
    }
}
