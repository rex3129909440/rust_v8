use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "outerText", get_value, set_value)
}

fn rendered_text(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    output: &mut String,
) {
    let Some(node_record) = super::node::record(scope, node) else {
        return;
    };
    if matches!(
        node_record.node_type,
        super::node::TEXT_NODE | super::node::CDATA_SECTION_NODE
    ) {
        output.push_str(node_record.node_value.as_deref().unwrap_or_default());
        return;
    }
    let tag =
        super::element::record(scope, node).map(|record| record.tag_name.to_ascii_uppercase());
    if tag.as_deref() == Some("BR") {
        output.push('\n');
        return;
    }
    if matches!(tag.as_deref(), Some("SCRIPT" | "STYLE" | "NOSCRIPT")) {
        return;
    }
    for child in super::node::children(scope, node) {
        rendered_text(scope, child, output);
    }
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut value = String::new();
    rendered_text(scope, arguments.this(), &mut value);
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn replacement_nodes<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner_document: Option<v8::Local<'s, v8::Object>>,
    value: &str,
) -> Vec<v8::Local<'s, v8::Object>> {
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    let runs = normalized.split('\n').collect::<Vec<_>>();
    let mut nodes = Vec::new();
    for (index, run) in runs.iter().enumerate() {
        if !run.is_empty()
            && let Ok(text) = super::text::create(scope, (*run).to_owned())
        {
            if let Some(document) = owner_document {
                super::node::set_owner_document(scope, text, document);
            }
            nodes.push(text);
        }
        if index + 1 < runs.len()
            && let Ok(line_break) = super::document::create_html_element_by_name(scope, "br")
        {
            if let Some(document) = owner_document {
                super::node::set_owner_document(scope, line_break, document);
            }
            nodes.push(line_break);
        }
    }
    nodes
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(parent) = super::node::parent(scope, arguments.this()) else {
        super::node::throw_dom_exception(
            scope,
            "NoModificationAllowedError",
            "The element has no parent.",
        );
        return;
    };
    let index = super::node::children(scope, parent)
        .iter()
        .position(|child| child.strict_equals(arguments.this().into()))
        .unwrap_or_else(|| super::node::children(scope, parent).len());
    let owner_document = super::node::record(scope, arguments.this())
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document));
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let nodes = replacement_nodes(scope, owner_document, &value);
    for (offset, node) in nodes.into_iter().enumerate() {
        if let Err((name, message)) = super::node::insert_node(scope, parent, node, index + offset)
        {
            super::node::throw_dom_exception(scope, name, message);
            return;
        }
    }
    super::node::detach(scope, arguments.this());
}
