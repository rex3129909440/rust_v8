use super::html_dialog_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "show", 0, show)
}

fn show(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.open {
        if current.modal {
            crate::webidl::throw_type_error(scope, "The dialog is already open as a modal dialog");
        }
        return;
    }
    update(scope, arguments.this(), |record| {
        record.open = true;
        record.modal = false;
    });
}
