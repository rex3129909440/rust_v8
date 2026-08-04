use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "snapshotLength", get_snapshot_length)
}

fn get_snapshot_length(
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
        wrong_result_type(scope, "snapshotLength");
        return;
    }
    let length = match record.payload {
        XPathPayload::Nodes(nodes) => nodes.len(),
        _ => 0,
    };
    result.set(v8::Integer::new_from_unsigned(scope, length as u32).into());
}
