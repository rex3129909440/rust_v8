pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "createEvent", 1, create_event)
}

fn create_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let interface_name = crate::webidl::value_to_string(scope, arguments.get(0));
    let android_touch_event = crate::browser_surface::current_version(scope).is_android()
        && (interface_name.eq_ignore_ascii_case("Touch")
            || interface_name.eq_ignore_ascii_case("TouchEvent"));
    let created = if android_touch_event {
        super::touch_event::create(scope)
    } else {
        match interface_name.as_str() {
            "BeforeUnloadEvent" => super::before_unload_event::create(scope),
            "CompositionEvent" => super::composition_event::create(scope),
            "CustomEvent" => super::custom_event::create(scope),
            "DeviceMotionEvent" => super::device_motion_event::create(scope),
            "DeviceOrientationEvent" => super::device_orientation_event::create(scope),
            "DragEvent" => super::drag_event::create(scope),
            "ErrorEvent" => {
                let error = v8::undefined(scope);
                super::error_event::create(scope, "", String::new(), error.into())
            }
            "Event" | "Events" | "HTMLEvents" => super::event::create(scope, ""),
            "FocusEvent" => super::focus_event::create(scope),
            "HashChangeEvent" => super::hash_change_event::create(scope),
            "KeyboardEvent" => super::keyboard_event::create(scope),
            "MessageEvent" => super::message_event::create_uninitialized(scope),
            "MouseEvent" | "MouseEvents" => super::mouse_event::create(scope, String::new()),
            "StorageEvent" => super::storage_event::create(scope),
            "TextEvent" => super::text_event::create(scope, String::new(), String::new()),
            "UIEvent" | "UIEvents" => super::ui_event::create(scope),
            _ => {
                super::node::throw_dom_exception(
                    scope,
                    "NotSupportedError",
                    &format!(
                        "Failed to execute 'createEvent' on 'Document': The provided event type ('{interface_name}') is invalid."
                    ),
                );
                return;
            }
        }
    };
    match created {
        Ok(event) => {
            super::event::mark_uninitialized(scope, event);
            result.set(event.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
