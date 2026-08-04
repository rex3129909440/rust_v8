use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "valueAsNumber",
        get_value_as_number,
        set_value_as_number,
    )
}

fn get_value_as_number(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, numeric_value(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_value_as_number(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let number = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !number.is_finite() {
        update(scope, a.this(), |x| x.value.clear());
        return;
    }
    let value = if current.input_type == "date" {
        let (year, month, day) = civil_from_days((number / 86_400_000.0).floor() as i64);
        format!("{year:04}-{month:02}-{day:02}")
    } else if matches!(current.input_type.as_str(), "number" | "range") {
        format_number(number)
    } else {
        crate::webidl::throw_type_error(
            scope,
            "valueAsNumber is not applicable to this input type",
        );
        return;
    };
    update(scope, a.this(), |x| x.value = value);
}
