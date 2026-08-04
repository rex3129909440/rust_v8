const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";
const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";
const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

pub(crate) fn parse_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    input: &str,
) -> Result<Vec<v8::Global<v8::Object>>, String> {
    let default_namespace = super::element::record(scope, context)
        .and_then(|record| record.namespace_uri)
        .unwrap_or_else(|| {
            if super::document::is_document(scope, context)
                && super::document::content_type(scope, context) != Some("text/html")
            {
                String::new()
            } else {
                HTML_NAMESPACE.to_owned()
            }
        });
    let mut roots = Vec::new();
    let mut stack: Vec<v8::Global<v8::Object>> = Vec::new();
    let mut position = 0;
    let html_mode = if super::document::is_document(scope, context) {
        super::document::content_type(scope, context) == Some("text/html")
    } else if let Some(document) =
        super::node::record(scope, context).and_then(|record| record.owner_document)
    {
        super::document::content_type(scope, v8::Local::new(scope, &document)) == Some("text/html")
    } else {
        default_namespace == HTML_NAMESPACE
    };
    if html_mode {
        return super::document_html_parser::parse_fragment(scope, context, input);
    }

    while position < input.len() {
        if input[position..].starts_with("<!--") {
            let end = input[position + 4..]
                .find("-->")
                .map_or(input.len(), |offset| position + 4 + offset);
            let comment = super::comment::create(scope, input[position + 4..end].to_owned())?;
            append_parsed_node(scope, &stack, &mut roots, comment)?;
            position = (end + usize::from(end < input.len()) * 3).min(input.len());
            continue;
        }
        if input[position..].starts_with("</") {
            let Some(relative_end) = input[position + 2..].find('>') else {
                append_text(scope, &stack, &mut roots, &input[position..])?;
                break;
            };
            let end = position + 2 + relative_end;
            let wanted = input[position + 2..end]
                .split_ascii_whitespace()
                .next()
                .unwrap_or_default();
            if let Some(index) = stack.iter().rposition(|element| {
                let element = v8::Local::new(scope, element);
                super::node::record(scope, element).is_some_and(|record| {
                    if html_mode {
                        record.node_name.eq_ignore_ascii_case(wanted)
                    } else {
                        record.node_name == wanted
                    }
                })
            }) {
                stack.truncate(index);
            }
            position = end + 1;
            continue;
        }
        if !html_mode && input[position..].starts_with("<![CDATA[") {
            let data_start = position + 9;
            let data_end = input[data_start..]
                .find("]]>")
                .map_or(input.len(), |offset| data_start + offset);
            let cdata =
                super::cdata_section::create(scope, input[data_start..data_end].to_owned())?;
            append_parsed_node(scope, &stack, &mut roots, cdata)?;
            position = (data_end + usize::from(data_end < input.len()) * 3).min(input.len());
            continue;
        }
        if !html_mode && input[position..].starts_with("<?") {
            let data_start = position + 2;
            let data_end = input[data_start..]
                .find("?>")
                .map_or(input.len(), |offset| data_start + offset);
            let body = input[data_start..data_end].trim();
            let target_end = body
                .find(|character: char| character.is_ascii_whitespace())
                .unwrap_or(body.len());
            let target = &body[..target_end];
            if !target.is_empty() && !target.eq_ignore_ascii_case("xml") {
                let data = body[target_end..].trim_start();
                let instruction = super::processing_instruction::create(
                    scope,
                    target.to_owned(),
                    data.to_owned(),
                )?;
                append_parsed_node(scope, &stack, &mut roots, instruction)?;
            }
            position = (data_end + usize::from(data_end < input.len()) * 2).min(input.len());
            continue;
        }
        if input[position..].starts_with("<!") || input[position..].starts_with("<?") {
            let end = input[position..]
                .find('>')
                .map_or(input.len(), |offset| position + offset + 1);
            position = end;
            continue;
        }
        if input.as_bytes()[position] == b'<' {
            let Some(end) = find_tag_end(input, position + 1) else {
                append_text(scope, &stack, &mut roots, &input[position..])?;
                break;
            };
            let mut body = input[position + 1..end].trim();
            let self_closing = body.ends_with('/');
            if self_closing {
                body = body[..body.len() - 1].trim_end();
            }
            let name_end = body
                .find(|character: char| character.is_ascii_whitespace())
                .unwrap_or(body.len());
            let raw_name = &body[..name_end];
            if raw_name.is_empty() || !valid_tag_name(raw_name) {
                append_text(scope, &stack, &mut roots, "<")?;
                position += 1;
                continue;
            }
            let parent = stack.last().map(|parent| v8::Local::new(scope, parent));
            let parent_namespace = parent
                .and_then(|parent| super::element::record(scope, parent))
                .and_then(|record| record.namespace_uri)
                .unwrap_or_else(|| default_namespace.clone());
            let parsed_attributes =
                parse_attributes(&body[name_end..], parent_namespace == HTML_NAMESPACE);
            let namespace = if html_mode {
                child_namespace(scope, parent, &parent_namespace, raw_name)
            } else {
                xml_element_namespace(scope, parent, raw_name, &parsed_attributes)
            };
            let normalized_name = normalized_tag_name(&namespace, raw_name);
            let element = create_element(scope, &namespace, &normalized_name)?;
            for (name, value) in &parsed_attributes {
                let namespace_uri = if html_mode {
                    attribute_namespace(name).map(str::to_owned)
                } else {
                    xml_attribute_namespace(scope, parent, name, &parsed_attributes)
                };
                super::element::set_attribute_full(
                    scope,
                    element,
                    name.clone(),
                    value.clone(),
                    namespace_uri,
                );
            }
            append_parsed_node(scope, &stack, &mut roots, element)?;
            position = end + 1;

            if !self_closing && !is_void_html_element(&namespace, &normalized_name) {
                if namespace == HTML_NAMESPACE && is_raw_text_element(&normalized_name) {
                    let closing = format!("</{normalized_name}");
                    if let Some(offset) = find_ascii_case_insensitive(&input[position..], &closing)
                    {
                        let raw_end = position + offset;
                        append_text_to(scope, element, &input[position..raw_end])?;
                        position = input[raw_end..]
                            .find('>')
                            .map_or(input.len(), |offset| raw_end + offset + 1);
                    } else {
                        append_text_to(scope, element, &input[position..])?;
                        position = input.len();
                    }
                } else {
                    stack.push(v8::Global::new(scope, element));
                }
            }
            continue;
        }

        let end = input[position..]
            .find('<')
            .map_or(input.len(), |offset| position + offset);
        append_text(scope, &stack, &mut roots, &input[position..end])?;
        position = end;
    }
    Ok(roots)
}

pub(crate) fn replace_children_with_html(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &str,
) -> Result<(), String> {
    let parsed = parse_fragment(scope, target, input)?;
    let target = template_contents(scope, target).unwrap_or(target);
    for child in super::node::children(scope, target) {
        super::node::detach(scope, child);
    }
    for (index, child) in parsed.iter().enumerate() {
        super::node::insert_node(scope, target, v8::Local::new(scope, child), index)
            .map_err(|(_, message)| message.to_owned())?;
    }
    Ok(())
}

pub(crate) fn serialize_children(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
) -> String {
    let parent = template_contents(scope, parent).unwrap_or(parent);
    let raw_text_parent = super::element::record(scope, parent).is_some_and(|record| {
        record.namespace_uri.as_deref() == Some(HTML_NAMESPACE)
            && is_raw_text_element(&record.tag_name)
    });
    super::node::children(scope, parent)
        .into_iter()
        .map(|child| {
            if raw_text_parent
                && super::node::record(scope, child).is_some_and(|record| record.node_type == 3)
            {
                super::character_data::data_if_character(scope, child).unwrap_or_default()
            } else {
                serialize_node(scope, child)
            }
        })
        .collect()
}

pub(crate) fn serialize_node(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
) -> String {
    let Some(record) = super::node::record(scope, node) else {
        return String::new();
    };
    match record.node_type {
        1 => serialize_element(scope, node),
        3 => {
            escape_text(&super::character_data::data_if_character(scope, node).unwrap_or_default())
        }
        4 => format!(
            "<![CDATA[{}]]>",
            super::character_data::data_if_character(scope, node).unwrap_or_default()
        ),
        7 => format!(
            "<?{}{}?>",
            record.node_name,
            super::character_data::data_if_character(scope, node)
                .filter(|data| !data.is_empty())
                .map(|data| format!(" {data}"))
                .unwrap_or_default()
        ),
        8 => format!(
            "<!--{}-->",
            super::character_data::data_if_character(scope, node).unwrap_or_default()
        ),
        9 | 11 => serialize_children(scope, node),
        10 => format!("<!DOCTYPE {}>", record.node_name),
        _ => String::new(),
    }
}

pub(crate) fn serialize_xml_node(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
) -> Result<String, String> {
    if super::node::record(scope, node).is_none() {
        return Err("The provided value is not a Node".to_owned());
    }
    let mut namespaces = std::collections::HashMap::new();
    namespaces.insert("xml".to_owned(), XML_NAMESPACE.to_owned());
    namespaces.insert("xmlns".to_owned(), XMLNS_NAMESPACE.to_owned());
    serialize_xml_node_with_namespaces(scope, node, &namespaces)
}

fn serialize_xml_node_with_namespaces(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    inherited_namespaces: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let record = super::node::record(scope, node)
        .ok_or_else(|| "The provided value is not a Node".to_owned())?;
    match record.node_type {
        1 => serialize_xml_element(scope, node, inherited_namespaces),
        3 => Ok(escape_xml_text(
            &super::character_data::data_if_character(scope, node).unwrap_or_default(),
        )),
        4 => {
            let data = super::character_data::data_if_character(scope, node).unwrap_or_default();
            if data.contains("]]>") {
                Err("A CDATASection may not contain ']]>'".to_owned())
            } else {
                Ok(format!("<![CDATA[{data}]]>"))
            }
        }
        7 => {
            let data = super::character_data::data_if_character(scope, node).unwrap_or_default();
            if data.contains("?>") {
                Err("Processing instruction data may not contain '?>'".to_owned())
            } else if data.is_empty() {
                Ok(format!("<?{}?>", record.node_name))
            } else {
                Ok(format!("<?{} {data}?>", record.node_name))
            }
        }
        8 => {
            let data = super::character_data::data_if_character(scope, node).unwrap_or_default();
            if data.contains("--") || data.ends_with('-') {
                Err("Comment data is not valid XML".to_owned())
            } else {
                Ok(format!("<!--{data}-->"))
            }
        }
        9 | 11 => {
            let mut output = String::new();
            for child in super::node::children(scope, node) {
                output.push_str(&serialize_xml_node_with_namespaces(
                    scope,
                    child,
                    inherited_namespaces,
                )?);
            }
            Ok(output)
        }
        10 => Ok(super::document_type::serialize(scope, node)
            .unwrap_or_else(|| format!("<!DOCTYPE {}>", record.node_name))),
        _ => Err("The node type cannot be XML serialized".to_owned()),
    }
}

fn serialize_xml_element(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    inherited_namespaces: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let record = super::element::record(scope, element)
        .ok_or_else(|| "The provided value is not an Element".to_owned())?;
    let attributes = super::element::attributes_snapshot(scope, element).unwrap_or_default();
    let serialized_name = if record.namespace_uri.as_deref() == Some(HTML_NAMESPACE) {
        record.tag_name.to_ascii_lowercase()
    } else {
        record.tag_name.clone()
    };
    let mut namespaces = inherited_namespaces.clone();
    for attribute in &attributes {
        if attribute.name == "xmlns" {
            namespaces.insert(String::new(), attribute.value.clone());
        } else if let Some(prefix) = attribute.name.strip_prefix("xmlns:") {
            namespaces.insert(prefix.to_owned(), attribute.value.clone());
        }
    }

    let mut generated_declarations = Vec::new();
    let element_prefix = record
        .tag_name
        .split_once(':')
        .map(|(prefix, _)| prefix)
        .unwrap_or_default();
    let element_namespace = record.namespace_uri.unwrap_or_default();
    ensure_xml_namespace_declaration(
        element_prefix,
        &element_namespace,
        &mut namespaces,
        &mut generated_declarations,
    );
    for attribute in &attributes {
        let Some((prefix, _)) = attribute.name.split_once(':') else {
            continue;
        };
        if prefix == "xmlns" {
            continue;
        }
        if let Some(namespace) = &attribute.namespace_uri {
            ensure_xml_namespace_declaration(
                prefix,
                namespace,
                &mut namespaces,
                &mut generated_declarations,
            );
        }
    }

    let mut output = String::new();
    output.push('<');
    output.push_str(&serialized_name);
    for (prefix, namespace) in generated_declarations {
        if prefix.is_empty() {
            output.push_str(" xmlns=\"");
        } else {
            output.push_str(" xmlns:");
            output.push_str(&prefix);
            output.push_str("=\"");
        }
        output.push_str(&escape_xml_attribute(&namespace));
        output.push('"');
    }
    for attribute in attributes {
        output.push(' ');
        output.push_str(&attribute.name);
        output.push_str("=\"");
        output.push_str(&escape_xml_attribute(&attribute.value));
        output.push('"');
    }
    let children = super::node::children(scope, element);
    if children.is_empty() {
        output.push_str("/>");
        return Ok(output);
    }
    output.push('>');
    for child in children {
        output.push_str(&serialize_xml_node_with_namespaces(
            scope,
            child,
            &namespaces,
        )?);
    }
    output.push_str("</");
    output.push_str(&serialized_name);
    output.push('>');
    Ok(output)
}

fn ensure_xml_namespace_declaration(
    prefix: &str,
    namespace: &str,
    namespaces: &mut std::collections::HashMap<String, String>,
    generated_declarations: &mut Vec<(String, String)>,
) {
    if namespaces.get(prefix).map(String::as_str) == Some(namespace) {
        return;
    }
    if prefix.is_empty() && namespace.is_empty() && !namespaces.contains_key(prefix) {
        namespaces.insert(String::new(), String::new());
        return;
    }
    namespaces.insert(prefix.to_owned(), namespace.to_owned());
    generated_declarations.push((prefix.to_owned(), namespace.to_owned()));
}

fn escape_xml_text(input: &str) -> String {
    input.replace('&', "&amp;").replace('<', "&lt;")
}

fn escape_xml_attribute(input: &str) -> String {
    escape_xml_text(input)
        .replace('"', "&quot;")
        .replace('\r', "&#13;")
        .replace('\n', "&#10;")
        .replace('\t', "&#9;")
}

fn serialize_element(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> String {
    let Some(record) = super::element::record(scope, element) else {
        return String::new();
    };
    let html = record.namespace_uri.as_deref() == Some(HTML_NAMESPACE);
    let name = if html {
        record.tag_name.to_ascii_lowercase()
    } else {
        record.tag_name
    };
    let mut output = String::with_capacity(name.len() * 2 + 5);
    output.push('<');
    output.push_str(&name);
    for attribute in super::element::attributes_snapshot(scope, element).unwrap_or_default() {
        output.push(' ');
        output.push_str(&attribute.name);
        output.push_str("=\"");
        output.push_str(&escape_attribute(&attribute.value));
        output.push('"');
    }
    output.push('>');
    if html && is_void_html_element(HTML_NAMESPACE, &name) {
        return output;
    }
    output.push_str(&serialize_children(scope, element));
    output.push_str("</");
    output.push_str(&name);
    output.push('>');
    output
}

fn template_contents<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::html_template_element::record(scope, element)
        .map(|record| v8::Local::new(scope, &record.content))
}

fn append_parsed_node(
    scope: &mut v8::PinScope<'_, '_>,
    stack: &[v8::Global<v8::Object>],
    roots: &mut Vec<v8::Global<v8::Object>>,
    node: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    if let Some(parent) = stack.last() {
        let parent = v8::Local::new(scope, parent);
        let index = super::node::children(scope, parent).len();
        super::node::insert_node(scope, parent, node, index)
            .map_err(|(_, message)| message.to_owned())
    } else {
        roots.push(v8::Global::new(scope, node));
        Ok(())
    }
}

fn append_text(
    scope: &mut v8::PinScope<'_, '_>,
    stack: &[v8::Global<v8::Object>],
    roots: &mut Vec<v8::Global<v8::Object>>,
    text: &str,
) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    let text = super::text::create(scope, decode_character_references(text))?;
    append_parsed_node(scope, stack, roots, text)
}

fn append_text_to(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    text: &str,
) -> Result<(), String> {
    if text.is_empty() {
        return Ok(());
    }
    let text = super::text::create(scope, text.to_owned())?;
    super::node::insert_node(
        scope,
        parent,
        text,
        super::node::children(scope, parent).len(),
    )
    .map_err(|(_, message)| message.to_owned())
}

fn create_element<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    namespace: &str,
    name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    match namespace {
        "" => super::element::create(scope, name.to_owned(), None),
        HTML_NAMESPACE => super::document::create_html_element_by_name(scope, name),
        SVG_NAMESPACE => super::document::create_svg_element(scope, name),
        MATHML_NAMESPACE => super::math_ml_element::create(scope, name.to_owned()),
        _ => super::element::create(scope, name.to_owned(), Some(namespace.to_owned())),
    }
}

fn child_namespace(
    scope: &v8::PinScope<'_, '_>,
    parent: Option<v8::Local<'_, v8::Object>>,
    parent_namespace: &str,
    raw_name: &str,
) -> String {
    if parent_namespace == HTML_NAMESPACE {
        if raw_name.eq_ignore_ascii_case("svg") {
            SVG_NAMESPACE.to_owned()
        } else if raw_name.eq_ignore_ascii_case("math") {
            MATHML_NAMESPACE.to_owned()
        } else {
            HTML_NAMESPACE.to_owned()
        }
    } else if parent_namespace == SVG_NAMESPACE
        && parent.is_some_and(|parent| {
            super::node::record(scope, parent)
                .is_some_and(|record| record.node_name.eq_ignore_ascii_case("foreignObject"))
        })
    {
        HTML_NAMESPACE.to_owned()
    } else {
        parent_namespace.to_owned()
    }
}

fn xml_element_namespace(
    scope: &v8::PinScope<'_, '_>,
    parent: Option<v8::Local<'_, v8::Object>>,
    qualified_name: &str,
    attributes: &[(String, String)],
) -> String {
    let prefix = qualified_name
        .split_once(':')
        .map(|(prefix, _)| prefix)
        .unwrap_or_default();
    if prefix == "xml" {
        return XML_NAMESPACE.to_owned();
    }
    xml_namespace_declaration(scope, parent, prefix, attributes).unwrap_or_default()
}

fn xml_attribute_namespace(
    scope: &v8::PinScope<'_, '_>,
    parent: Option<v8::Local<'_, v8::Object>>,
    qualified_name: &str,
    attributes: &[(String, String)],
) -> Option<String> {
    if qualified_name == "xmlns" || qualified_name.starts_with("xmlns:") {
        return Some(XMLNS_NAMESPACE.to_owned());
    }
    let (prefix, _) = qualified_name.split_once(':')?;
    if prefix == "xml" {
        return Some(XML_NAMESPACE.to_owned());
    }
    xml_namespace_declaration(scope, parent, prefix, attributes)
}

fn xml_namespace_declaration<'s>(
    scope: &v8::PinScope<'s, '_>,
    mut element: Option<v8::Local<'s, v8::Object>>,
    prefix: &str,
    attributes: &[(String, String)],
) -> Option<String> {
    let declaration_name = if prefix.is_empty() {
        "xmlns".to_owned()
    } else {
        format!("xmlns:{prefix}")
    };
    if let Some((_, value)) = attributes
        .iter()
        .rev()
        .find(|(name, _)| name == &declaration_name)
    {
        return (!value.is_empty()).then(|| value.clone());
    }
    while let Some(current) = element {
        if let Some(attribute) = super::element::attributes_snapshot(scope, current)
            .unwrap_or_default()
            .into_iter()
            .find(|attribute| attribute.name == declaration_name)
        {
            return (!attribute.value.is_empty()).then_some(attribute.value);
        }
        element = super::node::record(scope, current)
            .and_then(|record| record.parent)
            .map(|parent| v8::Local::new(scope, &parent));
    }
    None
}

fn normalized_tag_name(namespace: &str, raw_name: &str) -> String {
    if namespace == HTML_NAMESPACE {
        return raw_name.to_ascii_lowercase();
    }
    if namespace == SVG_NAMESPACE {
        return match raw_name.to_ascii_lowercase().as_str() {
            "animatemotion" => "animateMotion",
            "animatetransform" => "animateTransform",
            "clippath" => "clipPath",
            "feblend" => "feBlend",
            "fecolormatrix" => "feColorMatrix",
            "fecomponenttransfer" => "feComponentTransfer",
            "fecomposite" => "feComposite",
            "feconvolvematrix" => "feConvolveMatrix",
            "fediffuselighting" => "feDiffuseLighting",
            "fedisplacementmap" => "feDisplacementMap",
            "fedistantlight" => "feDistantLight",
            "fedropshadow" => "feDropShadow",
            "feflood" => "feFlood",
            "fefunca" => "feFuncA",
            "fefuncb" => "feFuncB",
            "fefuncg" => "feFuncG",
            "fefuncr" => "feFuncR",
            "fegaussianblur" => "feGaussianBlur",
            "feimage" => "feImage",
            "femerge" => "feMerge",
            "femergenode" => "feMergeNode",
            "femorphology" => "feMorphology",
            "feoffset" => "feOffset",
            "fepointlight" => "fePointLight",
            "fespecularlighting" => "feSpecularLighting",
            "fespotlight" => "feSpotLight",
            "fetile" => "feTile",
            "feturbulence" => "feTurbulence",
            "foreignobject" => "foreignObject",
            "lineargradient" => "linearGradient",
            "radialgradient" => "radialGradient",
            "textpath" => "textPath",
            other => other,
        }
        .to_owned();
    }
    raw_name.to_owned()
}

fn attribute_namespace(name: &str) -> Option<&'static str> {
    if name.eq_ignore_ascii_case("xmlns") || name.starts_with("xmlns:") {
        Some(XMLNS_NAMESPACE)
    } else if name.starts_with("xlink:") {
        Some(XLINK_NAMESPACE)
    } else if name.starts_with("xml:") {
        Some(XML_NAMESPACE)
    } else {
        None
    }
}

fn find_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, byte) in input.as_bytes()[start..].iter().copied().enumerate() {
        if matches!(byte, b'\'' | b'"') {
            if quote == Some(byte) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(byte);
            }
        } else if byte == b'>' && quote.is_none() {
            return Some(start + offset);
        }
    }
    None
}

fn parse_attributes(input: &str, lowercase_names: bool) -> Vec<(String, String)> {
    let bytes = input.as_bytes();
    let mut output: Vec<(String, String)> = Vec::new();
    let mut position = 0;
    while position < bytes.len() {
        while position < bytes.len() && bytes[position].is_ascii_whitespace() {
            position += 1;
        }
        if position >= bytes.len() {
            break;
        }
        let name_start = position;
        while position < bytes.len()
            && !bytes[position].is_ascii_whitespace()
            && bytes[position] != b'='
        {
            position += 1;
        }
        if name_start == position {
            position += 1;
            continue;
        }
        let raw_name = &input[name_start..position];
        let name = if lowercase_names {
            raw_name.to_ascii_lowercase()
        } else {
            raw_name.to_owned()
        };
        while position < bytes.len() && bytes[position].is_ascii_whitespace() {
            position += 1;
        }
        let mut value = String::new();
        if position < bytes.len() && bytes[position] == b'=' {
            position += 1;
            while position < bytes.len() && bytes[position].is_ascii_whitespace() {
                position += 1;
            }
            if position < bytes.len() && matches!(bytes[position], b'\'' | b'"') {
                let quote = bytes[position];
                position += 1;
                let value_start = position;
                while position < bytes.len() && bytes[position] != quote {
                    position += 1;
                }
                value = decode_character_references(&input[value_start..position]);
                if position < bytes.len() {
                    position += 1;
                }
            } else {
                let value_start = position;
                while position < bytes.len() && !bytes[position].is_ascii_whitespace() {
                    position += 1;
                }
                value = decode_character_references(&input[value_start..position]);
            }
        }
        if !output
            .iter()
            .any(|(existing, _)| existing.eq_ignore_ascii_case(&name))
        {
            output.push((name, value));
        }
    }
    output
}

fn decode_character_references(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;
    while let Some(offset) = remaining.find('&') {
        output.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
        let Some(end) = remaining.find(';') else {
            output.push_str(remaining);
            return output;
        };
        let entity = &remaining[1..end];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{a0}'),
            value if value.starts_with("#x") || value.starts_with("#X") => {
                u32::from_str_radix(&value[2..], 16)
                    .ok()
                    .and_then(char::from_u32)
            }
            value if value.starts_with('#') => {
                value[1..].parse::<u32>().ok().and_then(char::from_u32)
            }
            _ => None,
        };
        if let Some(decoded) = decoded {
            output.push(decoded);
        } else {
            output.push_str(&remaining[..=end]);
        }
        remaining = &remaining[end + 1..];
    }
    output.push_str(remaining);
    output
}

fn escape_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('\u{a0}', "&nbsp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attribute(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('\u{a0}', "&nbsp;")
        .replace('"', "&quot;")
}

fn valid_tag_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.'))
}

fn is_void_html_element(namespace: &str, name: &str) -> bool {
    namespace == HTML_NAMESPACE
        && matches!(
            name.to_ascii_lowercase().as_str(),
            "area"
                | "base"
                | "basefont"
                | "bgsound"
                | "br"
                | "col"
                | "embed"
                | "frame"
                | "hr"
                | "img"
                | "input"
                | "keygen"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
}

fn is_raw_text_element(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "script" | "style" | "xmp" | "iframe" | "noembed" | "noframes" | "plaintext"
    )
}

fn find_ascii_case_insensitive(input: &str, needle: &str) -> Option<usize> {
    input
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
