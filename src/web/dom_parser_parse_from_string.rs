use super::dom_parser::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "parseFromString", 2, parse_from_string)
}

fn parse_from_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !require_instance(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'parseFromString' on 'DOMParser': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let Some(source) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'parseFromString' on 'DOMParser'",
    ) else {
        return;
    };
    let Some(content_type) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(1),
        "Failed to execute 'parseFromString' on 'DOMParser'",
    ) else {
        return;
    };
    let document = match content_type.as_str() {
        "text/html" => {
            let title = html_title(&source);
            match super::html_document::create_from_source(scope, html_source(&source)) {
                Ok(document) => {
                    if let Some(title) = title {
                        super::document::set_string_value(scope, document, "title", title.trim());
                    }
                    Ok(document)
                }
                Err(message) => Err(message),
            }
        }
        "text/xml" => xml_document(scope, source, "text/xml"),
        "application/xml" => xml_document(scope, source, "application/xml"),
        "application/xhtml+xml" => xml_document(scope, source, "application/xhtml+xml"),
        "image/svg+xml" => xml_document(scope, source, "image/svg+xml"),
        _ => {
            crate::webidl::throw_type_error(
                scope,
                &format!(
                    "Failed to execute 'parseFromString' on 'DOMParser': The provided value '{content_type}' is not a valid enum value of type SupportedType."
                ),
            );
            return;
        }
    };
    match document {
        Ok(document) => result.set(document.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn xml_document<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: String,
    content_type: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let malformed = !xml_is_well_formed(&source);
    let document = super::xml_document::create_with_type(
        scope,
        if malformed {
            String::new()
        } else {
            source.clone()
        },
        content_type,
    )?;
    if !malformed {
        return Ok(document);
    }
    let parsed = super::dom_html::parse_fragment(scope, document, &source)?;
    for node in parsed {
        let node = v8::Local::new(scope, &node);
        if super::node::record(scope, node)
            .is_some_and(|record| record.node_type == super::node::ELEMENT_NODE)
        {
            super::node::set_owner_document_recursive(scope, node, document);
            super::node::insert_node(scope, document, node, 0)
                .map_err(|(_, message)| message.to_owned())?;
            break;
        }
    }
    let parser_error = super::element::create(
        scope,
        "parsererror".to_owned(),
        Some("http://www.mozilla.org/newlayout/xml/parsererror.xml".to_owned()),
    )?;
    let text = super::text::create(
        scope,
        "This page contains the following errors: error".to_owned(),
    )?;
    super::node::set_owner_document(scope, parser_error, document);
    super::node::set_owner_document(scope, text, document);
    super::node::insert_node(scope, parser_error, text, 0)
        .map_err(|(_, message)| message.to_owned())?;
    let parent = super::node::children(scope, document)
        .into_iter()
        .find(|node| {
            super::node::record(scope, *node)
                .is_some_and(|record| record.node_type == super::node::ELEMENT_NODE)
        })
        .unwrap_or(document);
    let index = super::node::children(scope, parent).len();
    super::node::insert_node(scope, parent, parser_error, index)
        .map_err(|(_, message)| message.to_owned())?;
    Ok(document)
}

fn xml_is_well_formed(source: &str) -> bool {
    let mut stack: Vec<&str> = Vec::new();
    let mut root_count = 0usize;
    let mut position = 0;
    while let Some(relative) = source[position..].find('<') {
        position += relative;
        let rest = &source[position..];
        if rest.starts_with("<!--") {
            let Some(end) = rest.find("-->") else {
                return false;
            };
            position += end + 3;
            continue;
        }
        if rest.starts_with("<![CDATA[") {
            let Some(end) = rest.find("]]>") else {
                return false;
            };
            position += end + 3;
            continue;
        }
        if rest.starts_with("<?") {
            let Some(end) = rest.find("?>") else {
                return false;
            };
            position += end + 2;
            continue;
        }
        let Some(end) = super::dom_html::find_markup_end(rest) else {
            return false;
        };
        let body = rest[1..end].trim();
        if body.starts_with('!') {
            position += end + 1;
            continue;
        }
        if let Some(closing) = body.strip_prefix('/') {
            let name = closing.split_ascii_whitespace().next().unwrap_or_default();
            if stack.pop() != Some(name) {
                return false;
            }
        } else {
            let name = body.split_ascii_whitespace().next().unwrap_or_default();
            if name.is_empty() {
                return false;
            }
            if stack.is_empty() {
                root_count += 1;
                if root_count > 1 {
                    return false;
                }
            }
            if !body.ends_with('/') {
                stack.push(name);
            }
        }
        position += end + 1;
    }
    stack.is_empty() && root_count == 1
}
