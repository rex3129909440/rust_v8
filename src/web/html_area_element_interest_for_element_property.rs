use super::html_area_element::*;

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
    match record(s, a.this()) {
        Some(x) => {
            if let Some(v) = x.interest_for {
                r.set(v8::Local::new(s, &v).into());
            } else if let Some(v) = super::element::reflected_element(s, a.this(), "interestfor") {
                r.set(v.into());
            } else {
                r.set(v8::null(s).into());
            }
        }
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
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
    let v = if a.get(0).is_null_or_undefined() {
        super::element::remove_attribute_value(s, a.this(), "interestfor");
        None
    } else if let Ok(o) = v8::Local::<v8::Object>::try_from(a.get(0)) {
        if super::element::record(s, o).is_none() {
            crate::webidl::throw_type_error(s, "The value must be an Element or null");
            return;
        }
        super::element::set_reflected_string(s, a.this(), "interestfor", String::new());
        Some(v8::Global::new(s, o))
    } else {
        crate::webidl::throw_type_error(s, "The value must be an Element or null");
        return;
    };
    update(s, a.this(), |x| x.interest_for = v)
}
