use std::collections::HashMap;

#[derive(Clone, Default)]
struct XsltProcessorRecord {
    stylesheet: Option<String>,
    parameters: HashMap<(String, String), v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct XsltProcessorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, XsltProcessorRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XsltProcessorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    if crate::browser_surface::restore_staged_window_property(scope, "XSLTProcessor")? {
        return Ok(());
    }
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XSLTProcessor", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XsltProcessorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XSLTProcessor",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "clearParameters", 0, clear_parameters)?;
    crate::webidl::define_method(scope, prototype, "getParameter", 2, get_parameter)?;
    crate::webidl::define_method(scope, prototype, "importStylesheet", 1, import_stylesheet)?;
    crate::webidl::define_method(scope, prototype, "removeParameter", 2, remove_parameter)?;
    crate::webidl::define_method(scope, prototype, "reset", 0, reset)?;
    crate::webidl::define_method(scope, prototype, "setParameter", 3, set_parameter)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "transformToDocument",
        1,
        transform_to_document,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "transformToFragment",
        2,
        transform_to_fragment,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XsltProcessorStore>()
        .ok_or_else(|| "XSLTProcessor state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'XSLTProcessor': Please use the 'new' operator.",
        );
        return;
    }
    scope
        .get_slot_mut::<XsltProcessorStore>()
        .expect("XSLTProcessor state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            XsltProcessorRecord::default(),
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<XsltProcessorRecord> {
    scope
        .get_slot::<XsltProcessorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut XsltProcessorRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<XsltProcessorStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(record);
    true
}

fn parameter_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> (String, String) {
    (
        crate::webidl::value_to_string(scope, arguments.get(0)),
        crate::webidl::value_to_string(scope, arguments.get(1)),
    )
}

fn clear_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| record.parameters.clear());
}

fn get_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let key = parameter_key(scope, &arguments);
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.parameters.get(&key) {
        Some(value) => result.set(v8::Local::new(scope, value)),
        None => result.set(v8::null(scope).into()),
    }
}

fn set_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 3 {
        crate::webidl::throw_type_error(scope, "setParameter requires 3 arguments");
        return;
    }
    let key = parameter_key(scope, &arguments);
    let value = v8::Global::new(scope, arguments.get(2));
    update(scope, arguments.this(), |record| {
        record.parameters.insert(key, value);
    });
}

fn remove_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let key = parameter_key(scope, &arguments);
    update(scope, arguments.this(), |record| {
        record.parameters.remove(&key);
    });
}

fn import_stylesheet(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "importStylesheet requires a Node");
        return;
    }
    let Ok(stylesheet) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The stylesheet must be a Node");
        return;
    };
    let Some(source) = super::document::serialize_if_document(scope, stylesheet) else {
        crate::webidl::throw_type_error(scope, "The stylesheet must be an XML Document");
        return;
    };
    update(scope, arguments.this(), |record| {
        record.stylesheet = Some(source)
    });
}

fn reset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.stylesheet = None;
        record.parameters.clear();
    });
}

fn source_document(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    index: i32,
) -> Option<String> {
    let object = v8::Local::<v8::Object>::try_from(arguments.get(index)).ok()?;
    super::document::serialize_if_document(scope, object)
}

fn transform_to_document(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(processor) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(source) = source_document(scope, &arguments, 0) else {
        crate::webidl::throw_type_error(scope, "transformToDocument requires a Node");
        return;
    };
    let output = apply_stylesheet(scope, &source, &processor);
    if let Ok(document) = super::xml_document::create_with_type(scope, output, "application/xml") {
        result.set(document.into());
    }
}

fn transform_to_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(processor) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(source) = source_document(scope, &arguments, 0) else {
        crate::webidl::throw_type_error(scope, "transformToFragment requires a Node");
        return;
    };
    let output = apply_stylesheet(scope, &source, &processor);
    let Ok(fragment) = super::document_fragment::create(scope) else {
        return;
    };
    if let Some(key) = v8::String::new(scope, "textContent")
        && let Some(value) = v8::String::new(scope, &strip_tags(&output))
    {
        let _ = fragment.set(scope, key.into(), value.into());
    }
    result.set(fragment.into());
}

fn apply_stylesheet(
    scope: &mut v8::PinScope<'_, '_>,
    source: &str,
    processor: &XsltProcessorRecord,
) -> String {
    let Some(stylesheet) = processor.stylesheet.as_ref() else {
        return source.to_owned();
    };
    let mut output = template_body(stylesheet).unwrap_or(stylesheet).to_owned();
    while let Some(start) = output.find("<xsl:value-of") {
        let Some(relative_end) = output[start..].find('>') else {
            break;
        };
        let end = start + relative_end + 1;
        let tag = &output[start..end];
        let select = attribute(tag, "select").unwrap_or_default();
        let value = if let Some(name) = select.strip_prefix('$') {
            processor
                .parameters
                .iter()
                .find(|((_, local), _)| local == name)
                .map(|(_, value)| {
                    crate::webidl::value_to_string(scope, v8::Local::new(scope, value))
                })
                .unwrap_or_default()
        } else {
            select_path(source, &select)
        };
        output.replace_range(start..end, &escape_xml(&value));
    }
    output
        .replace("</xsl:value-of>", "")
        .replace("<?xml version=\"1.0\"?>", "")
}

fn template_body(stylesheet: &str) -> Option<&str> {
    let start = stylesheet.find("<xsl:template")?;
    let open_end = stylesheet[start..].find('>')? + start + 1;
    let close = stylesheet[open_end..].find("</xsl:template>")? + open_end;
    Some(&stylesheet[open_end..close])
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let marker = format!("{name}=");
    let start = tag.find(&marker)? + marker.len();
    let quote = tag.as_bytes().get(start).copied()? as char;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + 1;
    let end = tag[value_start..].find(quote)? + value_start;
    Some(tag[value_start..end].to_owned())
}

fn select_path(source: &str, select: &str) -> String {
    let name = select
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .next_back()
        .unwrap_or(select);
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    source
        .find(&open)
        .and_then(|start| {
            let content_start = start + open.len();
            source[content_start..]
                .find(&close)
                .map(|end| source[content_start..content_start + end].to_owned())
        })
        .unwrap_or_default()
}

fn strip_tags(source: &str) -> String {
    let mut output = String::new();
    let mut inside = false;
    for character in source.chars() {
        match character {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => output.push(character),
            _ => {}
        }
    }
    output
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
