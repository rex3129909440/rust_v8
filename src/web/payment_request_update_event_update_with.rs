use super::payment_request_update_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "updateWith", 1, update_with)
}

fn update_with(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !is_instance(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let v = v8::Global::new(s, a.get(0));
    s.get_slot_mut::<PaymentRequestUpdateEventStore>()
        .expect("PaymentRequestUpdateEvent state")
        .updates
        .insert(a.this().get_identity_hash().get(), v);
}
