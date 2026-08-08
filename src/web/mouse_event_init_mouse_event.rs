use super::mouse_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "initMouseEvent", 1, init_mouse_event)
}

fn init_mouse_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let data = MouseEventData {
        bubbles: arguments.get(1).boolean_value(scope),
        cancelable: arguments.get(2).boolean_value(scope),
        view: (!arguments.get(3).is_null() && !arguments.get(3).is_undefined())
            .then(|| v8::Global::new(scope, arguments.get(3))),
        detail: arguments.get(4).int32_value(scope).unwrap_or(0),
        screen_x: arguments.get(5).int32_value(scope).unwrap_or(0),
        screen_y: arguments.get(6).int32_value(scope).unwrap_or(0),
        client_x: arguments.get(7).int32_value(scope).unwrap_or(0),
        client_y: arguments.get(8).int32_value(scope).unwrap_or(0),
        ctrl_key: arguments.get(9).boolean_value(scope),
        alt_key: arguments.get(10).boolean_value(scope),
        shift_key: arguments.get(11).boolean_value(scope),
        meta_key: arguments.get(12).boolean_value(scope),
        button: arguments.get(13).int32_value(scope).unwrap_or(0) as i16,
        related_target: (!arguments.get(14).is_null() && !arguments.get(14).is_undefined())
            .then(|| v8::Global::new(scope, arguments.get(14))),
        ..MouseEventData::default()
    };
    attach(scope, arguments.this(), event_type, data);
}
