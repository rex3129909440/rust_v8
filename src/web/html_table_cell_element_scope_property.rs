use super::html_table_cell_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "scope", get_string, set_string)
}

fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let raw = super::element::attribute_value(scope, arguments.this(), "scope").unwrap_or_default();
    let lower = raw.to_ascii_lowercase();
    let value = match lower.as_str() {
        "row" | "col" | "rowgroup" | "colgroup" => lower,
        _ => String::new(),
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_reflected_string(scope, arguments, "scope");
}
