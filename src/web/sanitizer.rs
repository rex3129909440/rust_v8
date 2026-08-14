use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq)]
struct SanitizerName {
    name: String,
    namespace: Option<String>,
}

#[derive(Clone)]
struct SanitizerConfig {
    elements: Option<Vec<SanitizerName>>,
    remove_elements: Vec<SanitizerName>,
    replace_with_children_elements: Vec<SanitizerName>,
    attributes: Option<Vec<SanitizerName>>,
    remove_attributes: Vec<SanitizerName>,
    processing_instructions: Option<Vec<SanitizerName>>,
    remove_processing_instructions: Vec<SanitizerName>,
    comments: Option<bool>,
    data_attributes: Option<bool>,
}

#[derive(Default)]
pub(crate) struct SanitizerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SanitizerConfig>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SanitizerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Sanitizer", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SanitizerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Sanitizer",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "allowAttribute", 1, allow_attribute)?;
    crate::webidl::define_method(scope, prototype, "allowElement", 1, allow_element)?;
    crate::webidl::define_method(scope, prototype, "get", 0, get)?;
    crate::webidl::define_method(scope, prototype, "removeAttribute", 1, remove_attribute)?;
    crate::webidl::define_method(scope, prototype, "removeElement", 1, remove_element)?;
    crate::webidl::define_method(scope, prototype, "removeUnsafe", 0, remove_unsafe)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "replaceElementWithChildren",
        1,
        replace_element_with_children,
    )?;
    crate::webidl::define_method(scope, prototype, "setComments", 1, set_comments)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setDataAttributes",
        1,
        set_data_attributes,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "allowProcessingInstruction",
        1,
        allow_processing_instruction,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "removeProcessingInstruction",
        1,
        remove_processing_instruction,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SanitizerStore>()
        .ok_or_else(|| "Sanitizer state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn html_name(name: &str) -> SanitizerName {
    SanitizerName {
        name: name.to_owned(),
        namespace: Some("http://www.w3.org/1999/xhtml".to_owned()),
    }
}

fn attribute_name(name: &str) -> SanitizerName {
    SanitizerName {
        name: name.to_owned(),
        namespace: None,
    }
}

fn default_config() -> SanitizerConfig {
    SanitizerConfig {
        elements: Some(vec![
            html_name("a"),
            html_name("abbr"),
            html_name("article"),
            html_name("b"),
            html_name("blockquote"),
            html_name("br"),
            html_name("code"),
            html_name("div"),
            html_name("em"),
            html_name("figure"),
            html_name("h1"),
            html_name("h2"),
            html_name("h3"),
            html_name("h4"),
            html_name("h5"),
            html_name("h6"),
            html_name("hr"),
            html_name("i"),
            html_name("li"),
            html_name("main"),
            html_name("mark"),
            html_name("ol"),
            html_name("p"),
            html_name("pre"),
            html_name("section"),
            html_name("small"),
            html_name("span"),
            html_name("strong"),
            html_name("sub"),
            html_name("sup"),
            html_name("table"),
            html_name("tbody"),
            html_name("td"),
            html_name("th"),
            html_name("thead"),
            html_name("tr"),
            html_name("u"),
            html_name("ul"),
        ]),
        remove_elements: Vec::new(),
        replace_with_children_elements: Vec::new(),
        attributes: Some(vec![
            attribute_name("class"),
            attribute_name("dir"),
            attribute_name("href"),
            attribute_name("id"),
            attribute_name("lang"),
            attribute_name("title"),
        ]),
        remove_attributes: Vec::new(),
        processing_instructions: Some(Vec::new()),
        remove_processing_instructions: Vec::new(),
        comments: Some(false),
        data_attributes: Some(false),
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'Sanitizer': Please use the 'new' operator.",
        );
        return;
    }
    let config = if arguments.get(0).is_undefined() {
        default_config()
    } else {
        let Ok(object) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'Sanitizer': Invalid Sanitizer configuration.",
            );
            return;
        };
        let elements = names_property(scope, object, "elements", true);
        let remove_elements =
            names_property(scope, object, "removeElements", true).unwrap_or_default();
        let replace_with_children_elements =
            names_property(scope, object, "replaceWithChildrenElements", true).unwrap_or_default();
        let attributes = names_property(scope, object, "attributes", false);
        let remove_attributes =
            names_property(scope, object, "removeAttributes", false).unwrap_or_default();
        let processing_instructions =
            names_property(scope, object, "processingInstructions", false);
        let remove_processing_instructions =
            names_property(scope, object, "removeProcessingInstructions", false)
                .unwrap_or_default();
        if (elements.is_some()
            && (!remove_elements.is_empty() || !replace_with_children_elements.is_empty()))
            || (attributes.is_some() && !remove_attributes.is_empty())
            || (processing_instructions.is_some() && !remove_processing_instructions.is_empty())
        {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'Sanitizer': Invalid Sanitizer configuration.",
            );
            return;
        }
        SanitizerConfig {
            elements,
            remove_elements,
            replace_with_children_elements,
            attributes,
            remove_attributes,
            processing_instructions,
            remove_processing_instructions,
            comments: bool_property(scope, object, "comments"),
            data_attributes: bool_property(scope, object, "dataAttributes"),
        }
    };
    scope
        .get_slot_mut::<SanitizerStore>()
        .expect("Sanitizer state")
        .records
        .insert(arguments.this().get_identity_hash().get(), config);
    result.set(arguments.this().into());
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn bool_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<bool> {
    let value = property(scope, object, name)?;
    (!value.is_undefined()).then(|| value.boolean_value(scope))
}

fn names_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    element: bool,
) -> Option<Vec<SanitizerName>> {
    let value = property(scope, object, name)?;
    if value.is_undefined() {
        return None;
    }
    let array = v8::Local::<v8::Array>::try_from(value).ok()?;
    let mut names = Vec::with_capacity(array.length() as usize);
    for index in 0..array.length() {
        if let Some(value) = array.get_index(scope, index) {
            if let Some(name) = parse_name(scope, value, element) {
                add_unique(&mut names, name);
            }
        }
    }
    Some(names)
}

fn parse_name(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    element: bool,
) -> Option<SanitizerName> {
    if value.is_string() {
        let name = crate::webidl::value_to_string(scope, value);
        return Some(if element {
            html_name(&name)
        } else {
            attribute_name(&name)
        });
    }
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let name_value = property(scope, object, "name")?;
    if name_value.is_undefined() {
        return None;
    }
    let name = crate::webidl::value_to_string(scope, name_value);
    let namespace = property(scope, object, "namespace").and_then(|value| {
        if value.is_null() || value.is_undefined() {
            None
        } else {
            Some(crate::webidl::value_to_string(scope, value))
        }
    });
    Some(SanitizerName { name, namespace })
}

fn add_unique(names: &mut Vec<SanitizerName>, name: SanitizerName) {
    if !names.contains(&name) {
        names.push(name);
    }
}

fn remove_name(names: &mut Vec<SanitizerName>, name: &SanitizerName) {
    names.retain(|entry| entry != name);
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut SanitizerConfig),
) -> bool {
    let Some(config) = scope
        .get_slot_mut::<SanitizerStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(config);
    true
}

fn require_receiver(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    let valid = scope.get_slot::<SanitizerStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    valid
}

fn require_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    method: &str,
    element: bool,
) -> Option<SanitizerName> {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'Sanitizer': 1 argument required, but only 0 present."
            ),
        );
        return None;
    }
    parse_name(scope, arguments.get(0), element)
}

fn allow_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_receiver(scope, arguments.this()) {
        return;
    }
    let Some(name) = require_name(scope, &arguments, "allowElement", true) else {
        return;
    };
    update(scope, arguments.this(), |config| {
        remove_name(&mut config.remove_elements, &name);
        remove_name(&mut config.replace_with_children_elements, &name);
        if let Some(elements) = config.elements.as_mut() {
            add_unique(elements, name);
        }
    });
}

fn remove_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_receiver(scope, arguments.this()) {
        return;
    }
    let Some(name) = require_name(scope, &arguments, "removeElement", true) else {
        return;
    };
    update(scope, arguments.this(), |config| {
        remove_name(&mut config.replace_with_children_elements, &name);
        if let Some(elements) = config.elements.as_mut() {
            remove_name(elements, &name);
        } else {
            add_unique(&mut config.remove_elements, name);
        }
    });
}

fn replace_element_with_children(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_receiver(scope, arguments.this()) {
        return;
    }
    let Some(name) = require_name(scope, &arguments, "replaceElementWithChildren", true) else {
        return;
    };
    update(scope, arguments.this(), |config| {
        remove_name(&mut config.remove_elements, &name);
        if let Some(elements) = config.elements.as_mut() {
            remove_name(elements, &name);
        }
        add_unique(&mut config.replace_with_children_elements, name);
    });
}

fn allow_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_receiver(scope, arguments.this()) {
        return;
    }
    let Some(name) = require_name(scope, &arguments, "allowAttribute", false) else {
        return;
    };
    update(scope, arguments.this(), |config| {
        remove_name(&mut config.remove_attributes, &name);
        if let Some(attributes) = config.attributes.as_mut() {
            add_unique(attributes, name);
        }
    });
}

fn remove_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_receiver(scope, arguments.this()) {
        return;
    }
    let Some(name) = require_name(scope, &arguments, "removeAttribute", false) else {
        return;
    };
    update(scope, arguments.this(), |config| {
        if let Some(attributes) = config.attributes.as_mut() {
            remove_name(attributes, &name);
        } else {
            add_unique(&mut config.remove_attributes, name);
        }
    });
}

fn allow_processing_instruction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_receiver(scope, arguments.this()) {
        return;
    }
    let Some(name) = require_name(scope, &arguments, "allowProcessingInstruction", false) else {
        return;
    };
    update(scope, arguments.this(), |config| {
        remove_name(&mut config.remove_processing_instructions, &name);
        if let Some(instructions) = config.processing_instructions.as_mut() {
            add_unique(instructions, name);
        }
    });
}

fn remove_processing_instruction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !require_receiver(scope, arguments.this()) {
        return;
    }
    let Some(name) = require_name(scope, &arguments, "removeProcessingInstruction", false) else {
        return;
    };
    update(scope, arguments.this(), |config| {
        if let Some(instructions) = config.processing_instructions.as_mut() {
            remove_name(instructions, &name);
        } else {
            add_unique(&mut config.remove_processing_instructions, name);
        }
    });
}

fn set_comments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |config| {
        config.comments = Some(value)
    });
}

fn set_data_attributes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |config| {
        config.data_attributes = Some(value)
    });
}

fn remove_unsafe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |config| {
        let unsafe_elements = [
            html_name("script"),
            html_name("iframe"),
            html_name("object"),
            html_name("embed"),
        ];
        for name in unsafe_elements {
            remove_name(&mut config.replace_with_children_elements, &name);
            if let Some(elements) = config.elements.as_mut() {
                remove_name(elements, &name);
            } else {
                add_unique(&mut config.remove_elements, name);
            }
        }
        let unsafe_attributes = [
            attribute_name("onclick"),
            attribute_name("onerror"),
            attribute_name("onload"),
        ];
        for name in unsafe_attributes {
            if let Some(attributes) = config.attributes.as_mut() {
                remove_name(attributes, &name);
            } else {
                add_unique(&mut config.remove_attributes, name);
            }
        }
    });
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(config) = scope
        .get_slot::<SanitizerStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    if let Some(attributes) = config.attributes.as_ref() {
        define_names(scope, output, "attributes", attributes);
    }
    if let Some(comments) = config.comments {
        define_bool(scope, output, "comments", comments);
    }
    if let Some(data_attributes) = config.data_attributes {
        define_bool(scope, output, "dataAttributes", data_attributes);
    }
    if let Some(elements) = config.elements.as_ref() {
        define_names(scope, output, "elements", elements);
    }
    if let Some(instructions) = config.processing_instructions.as_ref() {
        define_names(scope, output, "processingInstructions", instructions);
    }
    if !config.remove_attributes.is_empty() {
        define_names(scope, output, "removeAttributes", &config.remove_attributes);
    }
    if !config.remove_elements.is_empty() {
        define_names(scope, output, "removeElements", &config.remove_elements);
    }
    if !config.remove_processing_instructions.is_empty() {
        define_names(
            scope,
            output,
            "removeProcessingInstructions",
            &config.remove_processing_instructions,
        );
    }
    if !config.replace_with_children_elements.is_empty() {
        define_names(
            scope,
            output,
            "replaceWithChildrenElements",
            &config.replace_with_children_elements,
        );
    }
    result.set(output.into());
}

fn define_names(
    scope: &mut v8::PinScope<'_, '_>,
    output: v8::Local<'_, v8::Object>,
    property_name: &str,
    names: &[SanitizerName],
) {
    let array = v8::Array::new(scope, names.len() as i32);
    for (index, entry) in names.iter().enumerate() {
        let item = v8::Object::new(scope);
        define_text(scope, item, "name", &entry.name);
        match entry.namespace.as_ref() {
            Some(namespace) => define_text(scope, item, "namespace", namespace),
            None => define_value(scope, item, "namespace", v8::null(scope).into()),
        }
        let _ = array.set_index(scope, index as u32, item.into());
    }
    define_value(scope, output, property_name, array.into());
}

fn define_text(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define_value(scope, object, name, value.into());
    }
}

fn define_bool(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: bool,
) {
    define_value(scope, object, name, v8::Boolean::new(scope, value).into());
}

fn define_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::NONE);
    }
}
