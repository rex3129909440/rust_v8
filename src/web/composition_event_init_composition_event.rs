use super::composition_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "initCompositionEvent",
        1,
        init_composition_event,
    )
}

fn init_composition_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<CompositionEventStore>()
        .is_some_and(|store| {
            store
                .data
                .contains_key(&arguments.this().get_identity_hash().get())
        })
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let view = (!arguments.get(3).is_null_or_undefined())
        .then(|| v8::Global::new(scope, arguments.get(3)));
    let data = crate::webidl::value_to_string(scope, arguments.get(4));
    super::ui_event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
        view,
        0,
        None,
    );
    if let Some(current) = scope
        .get_slot_mut::<CompositionEventStore>()
        .and_then(|store| {
            store
                .data
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = data;
    }
}
