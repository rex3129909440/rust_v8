use std::ops::Deref;

use html5ever::tendril::TendrilSink;

const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";

struct ParsedAttribute {
    qualified_name: String,
    value: String,
    namespace_uri: Option<String>,
}

enum ParsedNode {
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    Comment(String),
    Text(String),
    Element {
        local_name: String,
        qualified_name: String,
        namespace_uri: Option<String>,
        attributes: Vec<ParsedAttribute>,
        children: Vec<ParsedNode>,
    },
    ProcessingInstruction {
        target: String,
        data: String,
    },
}

pub(crate) fn parse_page(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    source: &str,
) -> Result<(), String> {
    if !super::document::is_document(scope, document) {
        return Err("The page target is not a Document".to_owned());
    }
    let document = v8::Global::new(scope, document);
    let document = v8::Local::new(scope, &document);

    let parsed = scraper::Html::parse_document(source);
    let compat_mode = match parsed.quirks_mode {
        html5ever::tree_builder::QuirksMode::NoQuirks => "CSS1Compat",
        html5ever::tree_builder::QuirksMode::Quirks
        | html5ever::tree_builder::QuirksMode::LimitedQuirks => "BackCompat",
    };
    let nodes = parsed
        .tree
        .root()
        .children()
        .filter_map(snapshot_node)
        .collect::<Vec<_>>();

    for child in super::node::children(scope, document) {
        super::node::detach(scope, child);
    }
    for (index, node) in nodes.iter().enumerate() {
        let child = materialize_node(scope, document, node)?;
        if !super::node::insert_child(scope, document, child, index) {
            return Err("cannot connect parsed node to Document".to_owned());
        }
    }
    super::document::set_string_value(scope, document, "compatMode", compat_mode);
    Ok(())
}

pub(crate) fn parse_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    source: &str,
) -> Result<Vec<v8::Global<v8::Object>>, String> {
    let document = if super::document::is_document(scope, context) {
        context
    } else {
        super::node::owner_document(scope, context)
            .ok_or_else(|| "The fragment context has no owner document".to_owned())?
    };
    let document = v8::Global::new(scope, document);
    let (context_name, context_attributes) = fragment_context(scope, context);
    let parser = html5ever::driver::parse_fragment(
        scraper::HtmlTreeSink::new(scraper::Html::new_fragment()),
        Default::default(),
        context_name,
        context_attributes,
        true,
    );
    let parsed = parser.one(source);
    let fragment_root = parsed
        .tree
        .root()
        .children()
        .find(|node| {
            matches!(
                node.value(),
                scraper::Node::Element(element)
                    if element.name.ns == html5ever::ns!(html)
                        && element.name.local == html5ever::local_name!("html")
            )
        })
        .ok_or_else(|| "The HTML fragment parser did not produce a root element".to_owned())?;
    fragment_root
        .children()
        .filter_map(snapshot_node)
        .map(|node| {
            let document = v8::Local::new(scope, &document);
            materialize_node(scope, document, &node).map(|node| v8::Global::new(scope, node))
        })
        .collect()
}

fn fragment_context(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) -> (html5ever::QualName, Vec<html5ever::Attribute>) {
    let Some(record) = super::element::record(scope, context) else {
        return (
            html5ever::QualName::new(None, html5ever::ns!(html), html5ever::local_name!("body")),
            Vec::new(),
        );
    };
    let (prefix, local_name) = record
        .tag_name
        .split_once(':')
        .map_or((None, record.tag_name.as_str()), |(prefix, local_name)| {
            (Some(prefix), local_name)
        });
    let local_name = if record.namespace_uri.as_deref() == Some(HTML_NAMESPACE) {
        local_name.to_ascii_lowercase()
    } else {
        local_name.to_owned()
    };
    let name = html5ever::QualName::new(
        prefix.map(html5ever::Prefix::from),
        html5ever::Namespace::from(record.namespace_uri.as_deref().unwrap_or_default()),
        html5ever::LocalName::from(local_name.as_str()),
    );
    let attributes = super::element::attributes_snapshot(scope, context)
        .unwrap_or_default()
        .into_iter()
        .map(|attribute| {
            let (prefix, local_name) = attribute
                .name
                .split_once(':')
                .map_or((None, attribute.name.as_str()), |(prefix, local_name)| {
                    (Some(prefix), local_name)
                });
            html5ever::Attribute {
                name: html5ever::QualName::new(
                    prefix.map(html5ever::Prefix::from),
                    html5ever::Namespace::from(
                        attribute.namespace_uri.as_deref().unwrap_or_default(),
                    ),
                    html5ever::LocalName::from(local_name),
                ),
                value: attribute.value.into(),
            }
        })
        .collect();
    (name, attributes)
}

fn snapshot_node(node: ego_tree::NodeRef<'_, scraper::Node>) -> Option<ParsedNode> {
    let regular_children = || {
        node.children()
            .filter_map(snapshot_node)
            .collect::<Vec<_>>()
    };
    match node.value() {
        scraper::Node::Document | scraper::Node::Fragment => None,
        scraper::Node::Doctype(doctype) => Some(ParsedNode::Doctype {
            name: doctype.name().to_owned(),
            public_id: doctype.public_id().to_owned(),
            system_id: doctype.system_id().to_owned(),
        }),
        scraper::Node::Comment(comment) => Some(ParsedNode::Comment(comment.deref().to_owned())),
        scraper::Node::Text(text) => Some(ParsedNode::Text(text.deref().to_owned())),
        scraper::Node::Element(element) => {
            let namespace_uri = namespace(&element.name.ns);
            let element_qualified_name = qualified_name(
                element.name.prefix.as_ref().map(Deref::deref),
                element.name.local.deref(),
            );
            let attributes = element
                .attrs
                .iter()
                .map(|(name, value)| ParsedAttribute {
                    qualified_name: qualified_name(
                        name.prefix.as_ref().map(Deref::deref),
                        name.local.deref(),
                    ),
                    value: value.deref().to_owned(),
                    namespace_uri: namespace(&name.ns),
                })
                .collect();
            let children = if element.name.ns == html5ever::ns!(html)
                && element.name.local == html5ever::local_name!("template")
            {
                node.children()
                    .find(|child| matches!(child.value(), scraper::Node::Fragment))
                    .map(|content| {
                        content
                            .children()
                            .filter_map(snapshot_node)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            } else {
                regular_children()
            };
            Some(ParsedNode::Element {
                local_name: element.name.local.deref().to_owned(),
                qualified_name: element_qualified_name,
                namespace_uri,
                attributes,
                children,
            })
        }
        scraper::Node::ProcessingInstruction(instruction) => {
            Some(ParsedNode::ProcessingInstruction {
                target: instruction.target.deref().to_owned(),
                data: instruction.data.deref().to_owned(),
            })
        }
    }
}

fn materialize_node<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    document: v8::Local<'s, v8::Object>,
    parsed: &ParsedNode,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let node = match parsed {
        ParsedNode::Doctype {
            name,
            public_id,
            system_id,
        } => super::document_type::create(scope, name, public_id, system_id)?,
        ParsedNode::Comment(data) => super::comment::create(scope, data.clone())?,
        ParsedNode::Text(data) => super::text::create(scope, data.clone())?,
        ParsedNode::ProcessingInstruction { target, data } => {
            super::processing_instruction::create(scope, target.clone(), data.clone())?
        }
        ParsedNode::Element {
            local_name,
            qualified_name,
            namespace_uri,
            attributes,
            children,
        } => {
            let element =
                create_element(scope, local_name, qualified_name, namespace_uri.as_deref())?;
            if local_name.eq_ignore_ascii_case("script") {
                super::html_script_element::mark_parser_inserted(scope, element);
            }
            for attribute in attributes {
                if !super::element::set_attribute_full(
                    scope,
                    element,
                    attribute.qualified_name.clone(),
                    attribute.value.clone(),
                    attribute.namespace_uri.clone(),
                ) {
                    return Err(format!(
                        "cannot set parsed attribute {}",
                        attribute.qualified_name
                    ));
                }
            }
            let child_parent = if namespace_uri.as_deref() == Some(HTML_NAMESPACE)
                && local_name.eq_ignore_ascii_case("template")
            {
                let content = super::html_template_element::record(scope, element)
                    .ok_or_else(|| "HTMLTemplateElement state is missing".to_owned())?
                    .content;
                v8::Local::new(scope, &content)
            } else {
                element
            };
            for (index, child) in children.iter().enumerate() {
                let child = materialize_node(scope, document, child)?;
                if !super::node::insert_child(scope, child_parent, child, index) {
                    return Err("cannot connect parsed child node".to_owned());
                }
            }
            element
        }
    };
    super::node::set_owner_document(scope, node, document);
    Ok(node)
}

fn create_element<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    local_name: &str,
    qualified_name: &str,
    namespace_uri: Option<&str>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    match namespace_uri {
        Some(HTML_NAMESPACE) => {
            super::document::create_html_element_by_name(scope, &local_name.to_ascii_lowercase())
        }
        Some(SVG_NAMESPACE) => super::document::create_svg_element(scope, local_name),
        Some(MATHML_NAMESPACE) => super::math_ml_element::create(scope, local_name.to_owned()),
        namespace_uri => super::element::create(
            scope,
            qualified_name.to_owned(),
            namespace_uri.map(str::to_owned),
        ),
    }
}

fn namespace(namespace: &html5ever::Namespace) -> Option<String> {
    let namespace = namespace.deref();
    (!namespace.is_empty()).then(|| namespace.to_owned())
}

fn qualified_name(prefix: Option<&str>, local_name: &str) -> String {
    prefix.map_or_else(
        || local_name.to_owned(),
        |prefix| format!("{prefix}:{local_name}"),
    )
}
