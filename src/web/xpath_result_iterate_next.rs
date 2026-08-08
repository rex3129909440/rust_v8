use super::xpath_result::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "iterateNext", 0, iterate_next)
}

fn iterate_next(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(snapshot) = scope
        .get_slot::<XPathResultStore>()
        .and_then(|store| store.records.get(&id))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.result_type != UNORDERED_NODE_ITERATOR_TYPE
        && snapshot.result_type != ORDERED_NODE_ITERATOR_TYPE
    {
        wrong_result_type(scope, "iterateNext");
        return;
    }
    let node = match snapshot.payload {
        XPathPayload::Nodes(nodes) => nodes.get(snapshot.iterator_index).cloned(),
        _ => None,
    };
    if node.is_some() {
        if let Some(record) = scope
            .get_slot_mut::<XPathResultStore>()
            .and_then(|store| store.records.get_mut(&id))
        {
            record.iterator_index += 1;
        }
    }
    if let Some(node) = node {
        result.set(v8::Local::new(scope, &node).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
