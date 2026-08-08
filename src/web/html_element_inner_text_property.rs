use super::html_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "innerText", get_value, set_value)
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
    let block = matches!(
        tag.as_deref(),
        Some(
            "ADDRESS"
                | "ARTICLE"
                | "ASIDE"
                | "BLOCKQUOTE"
                | "DIV"
                | "DL"
                | "FIELDSET"
                | "FIGCAPTION"
                | "FIGURE"
                | "FOOTER"
                | "FORM"
                | "H1"
                | "H2"
                | "H3"
                | "H4"
                | "H5"
                | "H6"
                | "HEADER"
                | "HR"
                | "LI"
                | "MAIN"
                | "NAV"
                | "OL"
                | "P"
                | "PRE"
                | "SECTION"
                | "TABLE"
                | "TR"
                | "UL"
        )
    );
    if block && !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    for child in super::node::children(scope, node) {
        rendered_text(scope, child, output);
    }
    if block && !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
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
    for child in super::node::children(scope, arguments.this()) {
        rendered_text(scope, child, &mut value);
    }
    while value.ends_with('\n') {
        value.pop();
    }
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn insert_text_runs(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    value: &str,
) {
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    let owner_document = super::node::record(scope, parent)
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document));
    let runs = normalized.split('\n').collect::<Vec<_>>();
    for (index, run) in runs.iter().enumerate() {
        if !run.is_empty()
            && let Ok(text) = super::text::create(scope, (*run).to_owned())
        {
            if let Some(document) = owner_document {
                super::node::set_owner_document(scope, text, document);
            }
            let insertion = super::node::children(scope, parent).len();
            let _ = super::node::insert_node(scope, parent, text, insertion);
        }
        if index + 1 < runs.len()
            && let Ok(line_break) = super::document::create_html_element_by_name(scope, "br")
        {
            if let Some(document) = owner_document {
                super::node::set_owner_document(scope, line_break, document);
            }
            let insertion = super::node::children(scope, parent).len();
            let _ = super::node::insert_node(scope, parent, line_break, insertion);
        }
    }
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
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    for child in super::node::children(scope, arguments.this()) {
        super::node::detach(scope, child);
    }
    insert_text_runs(scope, arguments.this(), &value);
}
