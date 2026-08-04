use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "booleanValue", get_boolean_value)
}

fn get_boolean_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = require_record(scope, arguments.this()) else {
        return;
    };
    if record.result_type != BOOLEAN_TYPE {
        wrong_result_type(scope, "booleanValue");
        return;
    }
    let value = match record.payload {
        XPathPayload::Boolean(value) => value,
        XPathPayload::Number(value) => value != 0.0 && !value.is_nan(),
        XPathPayload::String(value) => !value.is_empty(),
        XPathPayload::Nodes(nodes) => !nodes.is_empty(),
    };
    result.set(v8::Boolean::new(scope, value).into());
}
