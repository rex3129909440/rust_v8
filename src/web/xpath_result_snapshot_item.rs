use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "snapshotItem", 1, snapshot_item)
}

fn snapshot_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = require_record(scope, arguments.this()) else {
        return;
    };
    if record.result_type != UNORDERED_NODE_SNAPSHOT_TYPE
        && record.result_type != ORDERED_NODE_SNAPSHOT_TYPE
    {
        wrong_result_type(scope, "snapshotItem");
        return;
    }
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let XPathPayload::Nodes(nodes) = record.payload {
        if let Some(node) = nodes.get(index) {
            result.set(v8::Local::new(scope, node).into());
            return;
        }
    }
    result.set(v8::null(scope).into());
}
