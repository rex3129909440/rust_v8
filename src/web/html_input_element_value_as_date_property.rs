use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "valueAsDate",
        get_value_as_date,
        set_value_as_date,
    )
}

fn get_value_as_date(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.input_type == "date" {
        let value = numeric_value(&record);
        if value.is_finite() {
            if let Some(date) = v8::Date::new(scope, value) {
                r.set(date.into());
                return;
            }
        }
    }
    r.set(v8::null(scope).into());
}

fn set_value_as_date(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.input_type != "date" {
        crate::webidl::throw_type_error(scope, "valueAsDate is not applicable to this input type");
        return;
    }
    if a.get(0).is_null() {
        update(scope, a.this(), |x| x.value.clear());
        return;
    }
    if !a.get(0).is_date() {
        crate::webidl::throw_type_error(scope, "valueAsDate must be a Date or null");
        return;
    }
    let millis = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !millis.is_finite() {
        update(scope, a.this(), |x| x.value.clear());
        return;
    }
    let (year, month, day) = civil_from_days((millis / 86_400_000.0).floor() as i64);
    let value = format!("{year:04}-{month:02}-{day:02}");
    update(scope, a.this(), |x| x.value = value);
}
