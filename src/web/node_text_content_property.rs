pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "textContent", get, set)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = super::node::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if matches!(
        record.node_type,
        super::node::DOCUMENT_NODE | super::node::DOCUMENT_TYPE_NODE
    ) {
        result.set(v8::null(scope).into());
    } else if let Some(value) =
        v8::String::new(scope, &super::node::node_text(scope, arguments.this()))
    {
        result.set(value.into());
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = super::node::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if matches!(
        record.node_type,
        super::node::DOCUMENT_NODE | super::node::DOCUMENT_TYPE_NODE
    ) {
        return;
    }
    let value = if arguments.get(0).is_null() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    if super::character_data::set_data_if_character(scope, arguments.this(), value.clone()) {
        return;
    }
    if record.node_type == super::node::ATTRIBUTE_NODE {
        super::attr::set_stored_value(scope, arguments.this(), value);
        return;
    }
    for child in super::node::children(scope, arguments.this()) {
        super::node::detach(scope, child);
    }
    if value.is_empty() {
        return;
    }
    let Ok(text) = super::text::create(scope, value) else {
        return;
    };
    if let Some(document) = record
        .owner_document
        .as_ref()
        .map(|document| v8::Local::new(scope, document))
    {
        super::node::set_owner_document(scope, text, document);
    }
    let _ = super::node::insert_node(scope, arguments.this(), text, 0);
}
