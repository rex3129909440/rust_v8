use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CssProperty {
    pub name: String,
    pub value: String,
    pub priority: String,
    pub source: String,
}

#[derive(Clone)]
struct CssStyleDeclarationRecord {
    properties: Vec<CssProperty>,
    parent_rule: Option<v8::Global<v8::Object>>,
    style_map: Option<v8::Global<v8::Object>>,
    owner_element: Option<v8::Global<v8::Object>>,
    readonly: bool,
}

#[derive(Default)]
pub(crate) struct CssStyleDeclarationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssStyleDeclarationRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssStyleDeclarationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSStyleDeclaration", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssStyleDeclarationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSStyleDeclaration",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "cssText", get_css_text, set_css_text)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "parentRule", get_parent_rule)?;
    crate::webidl::define_accessor(scope, prototype, "cssFloat", get_css_float, set_css_float)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getPropertyPriority",
        1,
        get_property_priority,
    )?;
    crate::webidl::define_method(scope, prototype, "getPropertyValue", 1, get_property_value)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::define_method(scope, prototype, "removeProperty", 1, remove_property)?;
    crate::webidl::define_method(scope, prototype, "setProperty", 2, set_property)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .ok_or_else(|| "CSSStyleDeclaration state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn normalize_name(name: &str) -> String {
    if name.eq_ignore_ascii_case("cssFloat") {
        "float".to_owned()
    } else if name.starts_with("--") {
        name.trim().to_owned()
    } else {
        name.trim().to_ascii_lowercase()
    }
}

// CSSOM property-name lookup is deliberately stricter than declaration
// parsing.  Edge treats ASCII case as insignificant for ordinary CSS
// property names, but it does not trim the argument and it keeps custom
// property names case-sensitive.  In particular, an unsupported token such
// as `ActiveBorder` is a color *value*, not a property name, so querying it
// must return the empty string even if malformed input reached the backing
// declaration record.
fn queried_property_name(name: &str) -> Option<String> {
    if name.trim() != name {
        return None;
    }
    if name.starts_with("--") {
        return (name.len() > 2).then(|| name.to_owned());
    }
    css_declaration_name(name)
}

pub(crate) fn parse_declarations(text: &str) -> Vec<CssProperty> {
    let mut properties = Vec::<CssProperty>::new();
    for declaration in text.split(';') {
        let Some((name, raw_value)) = declaration.split_once(':') else {
            continue;
        };
        let Some(name) = css_declaration_name(name) else {
            continue;
        };
        let mut value = raw_value.trim().to_owned();
        let mut priority = String::new();
        let lower = value.to_ascii_lowercase();
        if lower.ends_with("!important") {
            let end = value.len().saturating_sub("!important".len());
            value = value[..end].trim_end().to_owned();
            priority = "important".to_owned();
        }
        if value.is_empty() {
            continue;
        }
        let source = value.clone();
        let Some(value) = super::css_calculation::normalize_property_value(&name, &source) else {
            continue;
        };
        if let Some(existing) = properties.iter_mut().find(|entry| entry.name == name) {
            existing.value = value;
            existing.priority = priority;
            existing.source = source;
        } else {
            properties.push(CssProperty {
                name,
                value,
                priority,
                source,
            });
        }
    }
    properties
}

fn serialize_properties(properties: &[CssProperty]) -> String {
    let mut output = String::new();
    for property in properties {
        output.push_str(&property.name);
        output.push_str(": ");
        output.push_str(&property.value);
        if !property.priority.is_empty() {
            output.push_str(" !important");
        }
        output.push(';');
        output.push(' ');
    }
    output.trim_end().to_owned()
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    declarations: &str,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
    style_map: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let template = v8::ObjectTemplate::new(scope);
    template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(named_getter)
            .setter(named_setter)
            .query(named_query)
            .enumerator(named_enumerator)
            .descriptor(named_descriptor),
    );
    let object = template
        .new_instance(scope)
        .ok_or_else(|| "cannot create CSSStyleDeclaration exotic object".to_owned())?;
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSStyleDeclaration".to_owned());
    }
    attach(scope, object, declarations, parent_rule, style_map)?;
    Ok(object)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    declarations: &str,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
    style_map: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let properties = parse_declarations(declarations);
    let record = CssStyleDeclarationRecord {
        properties,
        parent_rule: parent_rule.map(|rule| v8::Global::new(scope, rule)),
        style_map: style_map.map(|map| v8::Global::new(scope, map)),
        owner_element: None,
        readonly: false,
    };
    scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .ok_or_else(|| "CSSStyleDeclaration state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    refresh_indexes(scope, object, 0);
    sync_style_map(scope, object);
    Ok(())
}

pub(crate) fn mark_readonly(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.readonly = true;
    true
}

fn reject_readonly(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    if record(scope, object).is_some_and(|record| record.readonly) {
        super::node::throw_dom_exception(
            scope,
            "NoModificationAllowedError",
            "These styles are computed, and therefore the 'style' property is read-only.",
        );
        true
    } else {
        false
    }
}

pub(crate) fn bind_owner(
    scope: &mut v8::PinScope<'_, '_>,
    declaration: v8::Local<'_, v8::Object>,
    owner: v8::Local<'_, v8::Object>,
) -> bool {
    let owner = v8::Global::new(scope, owner);
    let Some(record) = scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&declaration.get_identity_hash().get())
        })
    else {
        return false;
    };
    record.owner_element = Some(owner);
    true
}

pub(crate) fn property_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let name = queried_property_name(name)?;
    record(scope, object).map(|record| {
        record
            .properties
            .iter()
            .find(|property| property.name == name)
            .map(|property| property.value.clone())
            .unwrap_or_default()
    })
}

pub(crate) fn css_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| serialize_properties(&record.properties))
}

pub(crate) fn set_property_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: String,
) -> bool {
    if reject_readonly(scope, object) {
        return false;
    }
    let Some(name) = css_declaration_name(name) else {
        return false;
    };
    let source = value;
    let Some(value) = super::css_calculation::normalize_property_value(&name, &source) else {
        // Invalid CSS declarations are ignored and preserve the previous value.
        return true;
    };
    let Some(record) = scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    if value.is_empty() {
        record.properties.retain(|property| property.name != name);
    } else if let Some(property) = record
        .properties
        .iter_mut()
        .find(|property| property.name == name)
    {
        property.value = value;
        property.priority.clear();
        property.source = source;
    } else {
        record.properties.push(CssProperty {
            name,
            value,
            priority: String::new(),
            source,
        });
    }
    let new_length = record.properties.len();
    refresh_indexes(scope, object, new_length);
    sync_style_map(scope, object);
    sync_owner_attribute(scope, object);
    true
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssStyleDeclarationRecord> {
    scope
        .get_slot::<CssStyleDeclarationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn properties(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<CssProperty>> {
    Some(record(scope, object)?.properties)
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    Some(serialize_properties(&record(scope, object)?.properties))
}

fn refresh_indexes(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    previous_length: usize,
) {
    let Some(snapshot) = record(scope, object) else {
        return;
    };
    for index in 0..previous_length {
        let _ = object.delete_index(scope, index as u32);
    }
    for (index, property) in snapshot.properties.iter().enumerate() {
        if let Some(name) = v8::String::new(scope, &property.name) {
            let Some(key) = v8::String::new(scope, &index.to_string()) else {
                continue;
            };
            let _ = object.define_own_property(
                scope,
                key.into(),
                name.into(),
                v8::PropertyAttribute::READ_ONLY,
            );
        }
    }
}

fn sync_style_map(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let Some(snapshot) = record(scope, object) else {
        return;
    };
    let Some(style_map) = snapshot.style_map else {
        return;
    };
    let style_map = v8::Local::new(scope, &style_map);
    let mut values = Vec::<(String, v8::Global<v8::Value>)>::new();
    for property in &snapshot.properties {
        if let Ok(value) = super::css_style_value::create(scope, property.value.clone()) {
            let value: v8::Local<v8::Value> = value.into();
            values.push((property.name.clone(), v8::Global::new(scope, value)));
        }
    }
    super::style_property_map_read_only::update(scope, style_map, |record| {
        record.order.clear();
        record.values.clear();
        for (name, value) in values {
            record.order.push(name.clone());
            record.values.insert(name, vec![value]);
        }
    });
}

fn mutate(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Vec<CssProperty>),
) -> bool {
    if reject_readonly(scope, object) {
        return false;
    }
    let Some(previous_length) = scope
        .get_slot::<CssStyleDeclarationStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .map(|record| record.properties.len())
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    if let Some(record) = scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(&mut record.properties);
    }
    refresh_indexes(scope, object, previous_length);
    sync_style_map(scope, object);
    sync_owner_attribute(scope, object);
    true
}

fn replace_without_owner_sync(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    properties: Vec<CssProperty>,
) -> bool {
    let Some(previous_length) = scope
        .get_slot::<CssStyleDeclarationStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .map(|record| record.properties.len())
    else {
        return false;
    };
    if let Some(record) = scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.properties = properties;
    }
    refresh_indexes(scope, object, previous_length);
    sync_style_map(scope, object);
    true
}

pub(crate) fn set_text_from_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    text: &str,
) -> bool {
    replace_without_owner_sync(scope, object, parse_declarations(text))
}

fn sync_owner_attribute(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let Some(snapshot) = record(scope, object) else {
        return;
    };
    let Some(owner) = snapshot.owner_element else {
        return;
    };
    let owner = v8::Local::new(scope, &owner);
    let text = serialize_properties(&snapshot.properties);
    if text.is_empty() {
        super::element::remove_attribute_value(scope, owner, "style");
    } else {
        super::element::set_attribute_value(scope, owner, "style".to_owned(), text);
    }
}

fn get_css_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = serialize(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_css_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let properties = parse_declarations(&text);
    mutate(scope, arguments.this(), |current| *current = properties);
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.properties.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_parent_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.parent_rule {
            Some(rule) => result.set(v8::Local::new(scope, &rule).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn find_property(record: &CssStyleDeclarationRecord, name: &str) -> Option<CssProperty> {
    record
        .properties
        .iter()
        .find(|property| property.name == name)
        .cloned()
}

fn property_name(scope: &v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    if key.is_symbol() {
        return None;
    }
    let key = key.to_string(scope)?.to_rust_string_lossy(scope);
    css_named_property(&key)
}

fn css_named_property(name: &str) -> Option<String> {
    if matches!(
        name,
        "cssText"
            | "length"
            | "parentRule"
            | "item"
            | "getPropertyValue"
            | "getPropertyPriority"
            | "setProperty"
            | "removeProperty"
            | "constructor"
            | "toString"
            | "toLocaleString"
            | "valueOf"
            | "hasOwnProperty"
            | "isPrototypeOf"
            | "propertyIsEnumerable"
            | "__defineGetter__"
            | "__defineSetter__"
            | "__lookupGetter__"
            | "__lookupSetter__"
            | "__proto__"
    ) {
        return None;
    }
    if name == "cssFloat" {
        return Some("float".to_owned());
    }
    if !super::css_style_declaration_supported_properties::contains(name) {
        return None;
    }
    let mut output = if name.starts_with("webkit") {
        String::from("-")
    } else {
        String::new()
    };
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            output.push('-');
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    Some(output)
}

fn css_declaration_name(name: &str) -> Option<String> {
    let name = name.trim();
    if name.starts_with("--") {
        return (name.len() > 2).then(|| name.to_owned());
    }
    let name = name.to_ascii_lowercase();
    let mut idl_name = String::new();
    let mut uppercase_next = false;
    for character in name.trim_start_matches('-').chars() {
        if character == '-' {
            uppercase_next = true;
        } else if uppercase_next {
            idl_name.push(character.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            idl_name.push(character);
        }
    }
    super::css_style_declaration_supported_properties::contains(&idl_name).then_some(name)
}

pub(crate) fn supports_property(name: &str, value: &str) -> bool {
    let Some(name) = css_declaration_name(name) else {
        return false;
    };
    super::css_calculation::supports_property(&name, value)
}

fn named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "get", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = named_value(scope, arguments.holder(), &name) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = v8::String::new(scope, &value) else {
        return v8::Intercepted::kNo;
    };
    result.set(value.into());
    v8::Intercepted::kYes
}

fn named_setter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "set", key, Some(value));
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    let value = crate::webidl::value_to_string(scope, value);
    set_named_value(scope, arguments.holder(), &name, value);
    result.set_bool(true);
    v8::Intercepted::kYes
}

fn named_query(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "has", key, None);
    if property_name(scope, key).is_none() || record(scope, arguments.holder()).is_none() {
        return v8::Intercepted::kNo;
    }
    result.set_int32(0);
    v8::Intercepted::kYes
}

fn named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    if record(scope, arguments.holder()).is_none() {
        result.set(v8::Array::new(scope, 0));
        return;
    }
    let names = super::css_style_declaration_supported_properties::EDGE_150_SUPPORTED_PROPERTIES
        .iter()
        .filter_map(|name| v8::String::new(scope, name).map(|name| name.into()))
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &names));
}

fn named_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(
        scope,
        &arguments,
        "getOwnPropertyDescriptor",
        key,
        None,
    );
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = named_value(scope, arguments.holder(), &name) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = v8::String::new(scope, &value) else {
        return v8::Intercepted::kNo;
    };
    let descriptor = super::cross_origin_window_descriptors::data_descriptor(
        scope,
        value.into(),
        true,
        true,
        true,
    );
    result.set(descriptor.into());
    v8::Intercepted::kYes
}

pub(crate) fn named_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let name = normalize_name(name);
    Some(
        find_property(&record(scope, object)?, &name)
            .map(|property| property.value)
            .unwrap_or_default(),
    )
}

pub(crate) fn set_named_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: String,
) -> bool {
    let Some(name) = css_declaration_name(name) else {
        return false;
    };
    set_named_property(scope, object, name, value, String::new());
    true
}

fn get_property_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let requested_name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = queried_property_name(&requested_name)
        .and_then(|name| find_property(&record, &name))
        .map(|property| property.value)
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn get_property_priority(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let requested_name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = queried_property_name(&requested_name)
        .and_then(|name| find_property(&record, &name))
        .map(|property| property.priority)
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = record
        .properties
        .get(index)
        .map(|property| property.name.as_str())
        .unwrap_or("");
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn set_named_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: String,
    value: String,
    priority: String,
) {
    let source = value;
    let Some(value) = super::css_calculation::normalize_property_value(&name, &source) else {
        // Web CSSOM silently ignores an invalid assignment.
        return;
    };
    mutate(scope, object, |properties| {
        if value.is_empty() {
            properties.retain(|property| property.name != name);
        } else if let Some(property) = properties.iter_mut().find(|property| property.name == name)
        {
            property.value = value;
            property.priority = priority;
            property.source = source;
        } else {
            properties.push(CssProperty {
                name,
                value,
                priority,
                source,
            });
        }
    });
}

fn set_property(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let requested_name = crate::webidl::value_to_string(scope, arguments.get(0));
    let name = css_declaration_name(&requested_name);
    let value = crate::webidl::value_to_string(scope, arguments.get(1))
        .trim()
        .to_owned();
    let priority = if arguments.get(2).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(2)).to_ascii_lowercase()
    };
    let Some(name) = name else {
        return;
    };
    if !priority.is_empty() && priority != "important" {
        return;
    }
    set_named_property(scope, arguments.this(), name, value, priority);
}

fn remove_property(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let requested_name = crate::webidl::value_to_string(scope, arguments.get(0));
    let name = queried_property_name(&requested_name);
    let previous = record(scope, arguments.this())
        .and_then(|record| {
            name.as_deref()
                .and_then(|name| find_property(&record, name))
        })
        .map(|property| property.value)
        .unwrap_or_default();
    mutate(scope, arguments.this(), |properties| {
        if let Some(name) = &name {
            properties.retain(|property| property.name != *name);
        }
    });
    if let Some(previous) = v8::String::new(scope, &previous) {
        result.set(previous.into());
    }
}

fn get_css_float(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = find_property(&record, "float")
        .map(|property| property.value)
        .unwrap_or_default();
    let mut result = result;
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn set_text(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    text: &str,
) -> bool {
    let properties = parse_declarations(text);
    mutate(scope, object, |current| *current = properties)
}

fn set_css_float(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    set_named_property(
        scope,
        arguments.this(),
        "float".to_owned(),
        value,
        String::new(),
    );
}
