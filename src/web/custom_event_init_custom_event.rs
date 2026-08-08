use super::custom_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "initCustomEvent", 1, init_custom_event)
}

fn init_custom_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !scope.get_slot::<CustomEventStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&arguments.this().get_identity_hash().get())
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let detail = v8::Global::new(scope, arguments.get(3));
    super::event::reinitialize(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
    );
    if let Some(record) = scope.get_slot_mut::<CustomEventStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.detail = detail;
    }
}
