use super::ui_event::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "which", get_which)
}

fn get_which(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(keyboard) = super::keyboard_event::record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, keyboard.which).into());
    } else if super::pointer_event::record(scope, arguments.this()).is_some() {
        let which = super::mouse_event::record(scope, arguments.this()).map_or(0, |mouse| {
            match mouse.button {
                0 => 1,
                1 => 2,
                2 => 3,
                _ => 0,
            }
        });
        result.set(v8::Integer::new(scope, which).into());
    } else if let Some(mouse) = super::mouse_event::record(scope, arguments.this()) {
        let reports_button = super::event::record(scope, arguments.this())
            .is_some_and(|event| !event.is_trusted)
            || matches!(
                mouse.event_type.as_str(),
                "mousedown"
                    | "mouseup"
                    | "click"
                    | "auxclick"
                    | "dblclick"
                    | "contextmenu"
                    | "dragstart"
                    | "drag"
                    | "dragenter"
                    | "dragleave"
                    | "dragover"
                    | "drop"
                    | "dragend"
            );
        let which = if reports_button {
            match mouse.button {
                0 => 1,
                1 => 2,
                2 => 3,
                _ => 0,
            }
        } else {
            0
        };
        result.set(v8::Integer::new(scope, which).into());
    } else if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
