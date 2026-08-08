use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "popoverTargetElement",
        get_popover_target_element,
        set_popover_target_element,
    )
}

fn get_popover_target_element(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(target) = record.popover_target {
            r.set(v8::Local::new(scope, &target).into());
        } else if let Some(target) =
            super::element::reflected_element(scope, a.this(), "popovertarget")
        {
            r.set(target.into());
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_popover_target_element(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let target = if a.get(0).is_null_or_undefined() {
        super::element::remove_attribute_value(scope, a.this(), "popovertarget");
        None
    } else {
        let Ok(object) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
            crate::webidl::throw_type_error(
                scope,
                "popoverTargetElement must be an Element or null",
            );
            return;
        };
        if super::element::record(scope, object).is_none() {
            crate::webidl::throw_type_error(
                scope,
                "popoverTargetElement must be an Element or null",
            );
            return;
        }
        super::element::set_reflected_string(scope, a.this(), "popovertarget", String::new());
        Some(v8::Global::new(scope, object))
    };
    update(scope, a.this(), |x| x.popover_target = target);
}
