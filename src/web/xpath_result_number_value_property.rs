use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "numberValue", get_number_value)
}

fn get_number_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = require_record(scope, arguments.this()) else {
        return;
    };
    if record.result_type != NUMBER_TYPE {
        wrong_result_type(scope, "numberValue");
        return;
    }
    let value = match record.payload {
        XPathPayload::Number(value) => value,
        XPathPayload::String(value) => value.parse().unwrap_or(f64::NAN),
        XPathPayload::Boolean(value) => f64::from(value),
        XPathPayload::Nodes(nodes) => nodes.len() as f64,
    };
    result.set(v8::Number::new(scope, value).into());
}
