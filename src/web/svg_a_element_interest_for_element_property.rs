use super::svg_a_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "interestForElement",
        get_interest_for_element,
        set_interest_for_element,
    )
}

fn get_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(object) = v.interest_for_element {
            r.set(v8::Local::new(s, &object).into())
        } else if let Some(object) = super::element::reflected_element(s, a.this(), "interestfor") {
            r.set(object.into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn set_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = a.get(0);
    let object = if value.is_null_or_undefined() {
        super::element::remove_attribute_value(s, a.this(), "interestfor");
        None
    } else if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        if super::element::record(s, object).is_none() {
            crate::webidl::throw_type_error(s, "The value must be an Element or null");
            return;
        }
        super::element::set_reflected_string(s, a.this(), "interestfor", String::new());
        Some(v8::Global::new(s, object))
    } else {
        crate::webidl::throw_type_error(s, "The value must be an Element or null");
        return;
    };
    update(s, a.this(), |v| v.interest_for_element = object);
}
