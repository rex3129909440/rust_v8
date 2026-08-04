use super::ui_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "initUIEvent", 1, init_ui_event)
}

fn init_ui_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let view = (!arguments.get(3).is_null() && !arguments.get(3).is_undefined())
        .then(|| v8::Global::new(scope, arguments.get(3)));
    let detail = arguments.get(4).int32_value(scope).unwrap_or(0);
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
    );
    if let Some(record) = scope
        .get_slot_mut::<UiEventStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.view = view;
        record.detail = detail;
    }
}
