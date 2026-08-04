use super::message_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "initMessageEvent", 1, init_message_event)
}

fn init_message_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = arguments.get(1).boolean_value(scope);
    let cancelable = arguments.get(2).boolean_value(scope);
    let data = arguments.get(3);
    let origin = crate::webidl::value_to_string(scope, arguments.get(4));
    let last_event_id = crate::webidl::value_to_string(scope, arguments.get(5));
    let source = if arguments.get(6).is_null_or_undefined() {
        None
    } else if arguments.get(6).is_object() {
        Some(v8::Global::new(scope, arguments.get(6)))
    } else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'initMessageEvent': parameter 7 is not of type 'EventTarget'",
        );
        return;
    };
    let ports = read_ports(scope, arguments.get(7));
    attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        false,
        data,
        origin,
        last_event_id,
        source,
        ports,
        None,
    );
}
