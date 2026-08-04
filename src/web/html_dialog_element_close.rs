use super::html_dialog_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "close", 0, close)
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !current.open {
        return;
    }
    let return_value = if arguments.length() > 0 {
        Some(crate::webidl::value_to_string(scope, arguments.get(0)))
    } else {
        None
    };
    update(scope, arguments.this(), |record| {
        record.open = false;
        record.modal = false;
        if let Some(value) = return_value {
            record.return_value = value;
        }
    });
    let event = super::event_target::create_event(scope, "close");
    super::event_target::dispatch(scope, arguments.this(), event);
}
