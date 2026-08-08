use super::math_ml_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "oncancel", get_value, set_value)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.handlers.get("oncancel") {
        result.set(v8::Local::new(scope, handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    let present = handler.is_some();
    if let Some(record) = scope
        .get_slot_mut::<MathMlElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        if let Some(handler) = handler {
            record.handlers.insert("oncancel".to_owned(), handler);
        } else {
            record.handlers.remove("oncancel");
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::event_target::set_attribute_handler(scope, arguments.this(), "cancel", present);
}
