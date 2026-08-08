use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "singleNodeValue",
        get_single_node_value,
    )
}

fn get_single_node_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = require_record(scope, arguments.this()) else {
        return;
    };
    if record.result_type != ANY_UNORDERED_NODE_TYPE
        && record.result_type != FIRST_ORDERED_NODE_TYPE
    {
        wrong_result_type(scope, "singleNodeValue");
        return;
    }
    if let XPathPayload::Nodes(nodes) = record.payload {
        if let Some(node) = nodes.first() {
            result.set(v8::Local::new(scope, node).into());
            return;
        }
    }
    result.set(v8::null(scope).into());
}
