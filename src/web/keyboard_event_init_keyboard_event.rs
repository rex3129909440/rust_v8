use super::keyboard_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "initKeyboardEvent",
        1,
        init_keyboard_event,
    )
}

fn init_keyboard_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(mut record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.location = arguments.get(5).uint32_value(scope).unwrap_or(0);
    let modifiers = crate::webidl::value_to_string(scope, arguments.get(6));
    record.ctrl_key = modifiers.split_whitespace().any(|value| value == "Control");
    record.shift_key = modifiers.split_whitespace().any(|value| value == "Shift");
    record.alt_key = modifiers.split_whitespace().any(|value| value == "Alt");
    record.meta_key = modifiers.split_whitespace().any(|value| value == "Meta");
    record.repeat = false;
    super::ui_event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        arguments.get(1).boolean_value(scope),
        arguments.get(2).boolean_value(scope),
        false,
        Some(v8::Global::new(scope, arguments.get(3))),
        0,
        None,
    );
    if let Some(stored) = scope
        .get_slot_mut::<KeyboardEventStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *stored = record;
    }
}
