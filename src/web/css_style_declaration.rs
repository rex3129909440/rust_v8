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
        if name == "overflow" {
            // A shorthand resets both longhands at this point in declaration
            // order. Keeping an earlier overflow-x/y entry would make the
            // earlier declaration incorrectly outrank the later shorthand.
            properties.retain(|entry| !matches!(entry.name.as_str(), "overflow-x" | "overflow-y"));
        }
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
    template.set_indexed_property_handler(
        v8::IndexedPropertyHandlerConfiguration::new()
            .getter(indexed_getter)
            .query(indexed_query)
            .enumerator(indexed_enumerator),
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

pub(crate) fn create_readonly<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    properties: Vec<CssProperty>,
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
    template.set_indexed_property_handler(
        v8::IndexedPropertyHandlerConfiguration::new()
            .getter(indexed_getter)
            .query(indexed_query)
            .enumerator(indexed_enumerator),
    );
    let object = template
        .new_instance(scope)
        .ok_or_else(|| "cannot create CSSStyleDeclaration exotic object".to_owned())?;
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSStyleDeclaration".to_owned());
    }
    let record = CssStyleDeclarationRecord {
        properties,
        parent_rule: None,
        style_map: None,
        owner_element: None,
        readonly: true,
    };
    scope
        .get_slot_mut::<CssStyleDeclarationStore>()
        .ok_or_else(|| "CSSStyleDeclaration state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    refresh_indexes(scope, object, 0);
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
    let previous_length = {
        let Some(record) = scope
            .get_slot_mut::<CssStyleDeclarationStore>()
            .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
        else {
            return false;
        };
        record.readonly = true;
        record.properties.len()
    };
    refresh_indexes(scope, object, previous_length);
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
    if snapshot.readonly {
        return;
    }
    let indexed_names = snapshot
        .properties
        .iter()
        .map(|property| property.name.as_str())
        .collect::<Vec<_>>();
    for (index, property_name) in indexed_names.into_iter().enumerate() {
        if let Some(name) = v8::String::new(scope, property_name) {
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

fn indexed_getter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "get", index, None);
    if !record(scope, arguments.holder()).is_some_and(|record| record.readonly) {
        return v8::Intercepted::kNo;
    }
    let Some(name) =
        super::css_computed_style_properties::EDGE_150_COMPUTED_PROPERTIES.get(index as usize)
    else {
        return v8::Intercepted::kNo;
    };
    let Some(name) = v8::String::new(scope, name) else {
        return v8::Intercepted::kNo;
    };
    result.set(name.into());
    v8::Intercepted::kYes
}

fn indexed_query(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "has", index, None);
    if record(scope, arguments.holder()).is_some_and(|record| record.readonly)
        && (index as usize)
            < super::css_computed_style_properties::EDGE_150_COMPUTED_PROPERTIES.len()
    {
        result.set_int32(1);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn indexed_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let length = if record(scope, arguments.holder()).is_some_and(|record| record.readonly) {
        super::css_computed_style_properties::EDGE_150_COMPUTED_PROPERTIES.len()
    } else {
        0
    };
    let indices = (0..length)
        .map(|index| v8::Integer::new_from_unsigned(scope, index as u32).into())
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &indices));
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
    if record(scope, arguments.this()).is_some_and(|record| record.readonly) {
        result.set(v8::String::empty(scope).into());
    } else if let Some(value) = serialize(scope, arguments.this()) {
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
        let length = if record.readonly {
            super::css_computed_style_properties::EDGE_150_COMPUTED_PROPERTIES.len()
        } else {
            record.properties.len()
        };
        result.set(v8::Integer::new_from_unsigned(scope, length as u32).into());
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

fn find_property_in(properties: &[CssProperty], name: &str) -> Option<String> {
    properties
        .iter()
        .find(|property| property.name == name)
        .map(|property| property.value.clone())
}

pub(crate) fn computed_value_from_properties(properties: &[CssProperty], name: &str) -> String {
    find_property_in(properties, name)
        .or_else(|| computed_shorthand(properties, name))
        .unwrap_or_default()
}

fn computed_shorthand(properties: &[CssProperty], name: &str) -> Option<String> {
    match name {
        "font" => computed_font_shorthand(properties),
        "margin" | "padding" | "border-width" | "border-style" | "border-color" => {
            let stem = name.strip_prefix("border-");
            let property = |side: &str| match stem {
                Some(suffix) => format!("border-{side}-{suffix}"),
                None => format!("{name}-{side}"),
            };
            let default = match name {
                "border-style" => "none",
                "border-color" => "rgb(0, 0, 0)",
                _ => "0px",
            };
            let top = find_property_in(properties, &property("top"))
                .unwrap_or_else(|| default.to_owned());
            let right = find_property_in(properties, &property("right"))
                .unwrap_or_else(|| default.to_owned());
            let bottom = find_property_in(properties, &property("bottom"))
                .unwrap_or_else(|| default.to_owned());
            let left = find_property_in(properties, &property("left"))
                .unwrap_or_else(|| default.to_owned());
            Some(serialize_quad(&top, &right, &bottom, &left))
        }
        _ => None,
    }
}

fn computed_initial_longhand(name: &str) -> Option<&'static str> {
    match name {
        "padding-top"
        | "padding-right"
        | "padding-bottom"
        | "padding-left"
        | "margin-top"
        | "margin-right"
        | "margin-bottom"
        | "margin-left"
        | "border-top-width"
        | "border-right-width"
        | "border-bottom-width"
        | "border-left-width" => Some("0px"),
        "border-top-style" | "border-right-style" | "border-bottom-style" | "border-left-style" => {
            Some("none")
        }
        "border-top-color" | "border-right-color" | "border-bottom-color" | "border-left-color" => {
            Some("rgb(0, 0, 0)")
        }
        _ => None,
    }
}

fn serialize_quad(top: &str, right: &str, bottom: &str, left: &str) -> String {
    if top == right && top == bottom && top == left {
        top.to_owned()
    } else if top == bottom && right == left {
        format!("{top} {right}")
    } else if right == left {
        format!("{top} {right} {bottom}")
    } else {
        format!("{top} {right} {bottom} {left}")
    }
}

fn computed_font_shorthand(properties: &[CssProperty]) -> Option<String> {
    let style = find_property_in(properties, "font-style").unwrap_or_else(|| "normal".to_owned());
    let variant =
        find_property_in(properties, "font-variant").unwrap_or_else(|| "normal".to_owned());
    let weight = find_property_in(properties, "font-weight").unwrap_or_else(|| "400".to_owned());
    let stretch = find_property_in(properties, "font-stretch").unwrap_or_else(|| "100%".to_owned());
    let size = find_property_in(properties, "font-size")?;
    let line_height =
        find_property_in(properties, "line-height").unwrap_or_else(|| "normal".to_owned());
    let family = find_property_in(properties, "font-family")?;
    let mut components = Vec::new();
    if style != "normal" {
        components.push(style);
    }
    if variant != "normal" {
        components.push(variant);
    }
    if weight != "normal" && weight != "400" {
        components.push(weight);
    }
    if stretch != "normal" && stretch != "100%" {
        components.push(stretch);
    }
    if line_height == "normal" {
        components.push(size);
    } else {
        components.push(format!("{size} / {line_height}"));
    }
    components.push(family);
    Some(components.join(" "))
}

fn property_name(scope: &v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    if key.is_symbol() {
        return None;
    }
    let key = key.to_string(scope)?.to_rust_string_lossy(scope);
    css_named_property(&key)
}

fn css_named_property(name: &str) -> Option<String> {
    // Blink's legacy own-key enumerator still lists these historical EPUB
    // aliases, but the named-property getter/query does not expose them.
    // Consequently Reflect.ownKeys() includes them while
    // getOwnPropertyDescriptor() and Object.keys() omit them.
    if matches!(
        name,
        "epubCaptionSide"
            | "epubTextCombine"
            | "epubTextEmphasis"
            | "epubTextEmphasisColor"
            | "epubTextEmphasisStyle"
            | "epubTextOrientation"
            | "epubTextTransform"
            | "epubWordBreak"
            | "epubWritingMode"
    ) {
        return None;
    }
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
    // Blink exposes the legacy IDL spelling as an alias of the unprefixed
    // property. Both accessors read and write the same declaration slot.
    if name == "webkitTextSizeAdjust" {
        return Some("text-size-adjust".to_owned());
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
    let record = record(scope, object)?;
    Some(
        find_property(&record, &name)
            .map(|property| property.value)
            .or_else(|| {
                find_property(&record, "font").and_then(|font| {
                    font_longhands_from_shorthand(&font.source)
                        .into_iter()
                        .find(|property| property.name == name)
                        .map(|property| property.value)
                })
            })
            .or_else(|| {
                record
                    .readonly
                    .then(|| computed_shorthand(&record.properties, &name))
                    .flatten()
            })
            .or_else(|| {
                record
                    .readonly
                    .then(|| computed_initial_longhand(&name).map(str::to_owned))
                    .flatten()
            })
            .unwrap_or_default(),
    )
}

pub(crate) fn font_longhands_from_shorthand(value: &str) -> Vec<CssProperty> {
    let tokens = split_font_shorthand_tokens(value);
    let Some(size_index) = tokens.iter().position(|token| is_font_size_token(token)) else {
        return Vec::new();
    };
    let mut style = "normal".to_owned();
    let mut variant = "normal".to_owned();
    let mut weight = "400".to_owned();
    let mut stretch = "100%".to_owned();
    for token in &tokens[..size_index] {
        let lower = token.to_ascii_lowercase();
        if matches!(lower.as_str(), "italic" | "oblique") || lower.starts_with("oblique(") {
            style = token.clone();
        } else if lower == "small-caps" {
            variant = token.clone();
        } else if matches!(
            lower.as_str(),
            "bold"
                | "bolder"
                | "lighter"
                | "100"
                | "200"
                | "300"
                | "400"
                | "500"
                | "600"
                | "700"
                | "800"
                | "900"
        ) {
            weight = if lower == "bold" {
                "700".to_owned()
            } else {
                token.clone()
            };
        } else if matches!(
            lower.as_str(),
            "ultra-condensed"
                | "extra-condensed"
                | "condensed"
                | "semi-condensed"
                | "semi-expanded"
                | "expanded"
                | "extra-expanded"
                | "ultra-expanded"
        ) {
            stretch = token.clone();
        }
    }
    let size = tokens[size_index].clone();
    let mut family_index = size_index + 1;
    let mut line_height = "normal".to_owned();
    if tokens.get(family_index).is_some_and(|token| token == "/") {
        if let Some(value) = tokens.get(family_index + 1) {
            line_height = value.clone();
            family_index += 2;
        }
    }
    let family = tokens[family_index..].join(" ");
    if family.is_empty() {
        return Vec::new();
    }
    [
        ("font-style", style),
        ("font-variant", variant),
        ("font-weight", weight),
        ("font-stretch", stretch),
        ("font-size", size),
        ("line-height", line_height),
        ("font-family", family),
    ]
    .into_iter()
    .map(|(name, value)| CssProperty {
        name: name.to_owned(),
        source: value.clone(),
        value,
        priority: String::new(),
    })
    .collect()
}

fn split_font_shorthand_tokens(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut depth = 0_u32;
    for character in value.chars() {
        if let Some(active) = quote {
            current.push(character);
            if character == active {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => {
                quote = Some(character);
                current.push(character);
            }
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            '/' if depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_owned());
                }
                current.clear();
                tokens.push("/".to_owned());
            }
            character if character.is_whitespace() && depth == 0 => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_owned());
                    current.clear();
                }
            }
            _ => current.push(character),
        }
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_owned());
    }
    tokens
}

fn is_font_size_token(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "xx-small"
            | "x-small"
            | "small"
            | "medium"
            | "large"
            | "x-large"
            | "xx-large"
            | "xxx-large"
            | "smaller"
            | "larger"
    ) || lower.starts_with("calc(")
        || lower.starts_with("min(")
        || lower.starts_with("max(")
        || lower.starts_with("clamp(")
    {
        return true;
    }
    const UNITS: &[&str] = &[
        "px", "pt", "pc", "in", "cm", "mm", "q", "em", "rem", "ex", "rex", "ch", "rch", "cap",
        "rcap", "ic", "ric", "lh", "rlh", "vw", "vh", "vmin", "vmax", "vi", "vb", "svw", "svh",
        "lvw", "lvh", "dvw", "dvh", "%",
    ];
    UNITS.iter().any(|unit| {
        lower
            .strip_suffix(unit)
            .is_some_and(|number| number.trim().parse::<f64>().is_ok())
    })
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
        .map(|name| {
            find_property(&record, &name)
                .map(|property| property.value)
                .or_else(|| {
                    record.readonly.then(|| {
                        if name == "background" {
                            Some(computed_background_shorthand(&record))
                        } else {
                            computed_shorthand(&record.properties, &name)
                                .or_else(|| computed_initial_longhand(&name).map(str::to_owned))
                        }
                    })?
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn computed_background_shorthand(record: &CssStyleDeclarationRecord) -> String {
    let value = |name: &str, default: &str| {
        find_property(record, name)
            .map(|property| property.value)
            .unwrap_or_else(|| default.to_owned())
    };
    let color = value("background-color", "rgba(0, 0, 0, 0)");
    let image = value("background-image", "none");
    let repeat_x = value("background-repeat-x", "repeat");
    let repeat_y = value("background-repeat-y", "repeat");
    let repeat = if repeat_x == repeat_y {
        repeat_x
    } else {
        format!("{repeat_x} {repeat_y}")
    };
    let attachment = value("background-attachment", "scroll");
    let position_x = value("background-position-x", "0%");
    let position_y = value("background-position-y", "0%");
    let size = value("background-size", "auto");
    let origin = value("background-origin", "padding-box");
    let clip = value("background-clip", "border-box");
    format!(
        "{color} {image} {repeat} {attachment} {position_x} {position_y} / {size} {origin} {clip}"
    )
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
    let value = if record.readonly {
        super::css_computed_style_properties::EDGE_150_COMPUTED_PROPERTIES
            .get(index)
            .copied()
            .unwrap_or("")
    } else {
        record
            .properties
            .get(index)
            .map(|property| property.name.as_str())
            .unwrap_or("")
    };
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
        if name == "overflow" && !value.is_empty() {
            properties
                .retain(|property| !matches!(property.name.as_str(), "overflow-x" | "overflow-y"));
        }
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
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
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
