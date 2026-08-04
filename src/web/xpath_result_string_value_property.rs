use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "stringValue", get_string_value)
}

fn get_string_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = require_record(scope, arguments.this()) else {
        return;
    };
    if record.result_type != STRING_TYPE {
        wrong_result_type(scope, "stringValue");
        return;
    }
    let value = match record.payload {
        XPathPayload::String(value) => value,
        XPathPayload::Number(value) => value.to_string(),
        XPathPayload::Boolean(value) => value.to_string(),
        XPathPayload::Nodes(nodes) => nodes.len().to_string(),
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}
