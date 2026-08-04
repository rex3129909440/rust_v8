use super::mouse_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getModifierState", 1, get_modifier_state)
}

fn get_modifier_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let modifier = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let active = match modifier.as_str() {
        "Control" => record.ctrl_key,
        "Shift" => record.shift_key,
        "Alt" => record.alt_key,
        "Meta" => record.meta_key,
        _ => false,
    };
    result.set(v8::Boolean::new(scope, active).into());
}
