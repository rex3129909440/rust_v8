use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DomStringMapStore {
    constructor: crate::webidl::RealmConstructor,
    owners: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomStringMapStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMStringMap", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DomStringMapStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMStringMap",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomStringMapStore>()
        .ok_or_else(|| "DOMStringMap state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let template = v8::ObjectTemplate::new(scope);
    template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(named_getter)
            .setter(named_setter)
            .query(named_query)
            .deleter(named_deleter)
            .enumerator(named_enumerator),
    );
    let map = template
        .new_instance(scope)
        .ok_or_else(|| "cannot create DOMStringMap exotic object".to_owned())?;
    if crate::webidl::set_platform_prototype(scope, map, prototype.into()) != Some(true) {
        return Err("cannot create DOMStringMap".to_owned());
    }
    let owner = v8::Global::new(scope, owner);
    scope
        .get_slot_mut::<DomStringMapStore>()
        .ok_or_else(|| "DOMStringMap state was not prepared".to_owned())?
        .owners
        .insert(map.get_identity_hash().get(), owner);
    Ok(map)
}

pub(crate) fn dataset_name(attribute_name: &str) -> String {
    let source = attribute_name
        .strip_prefix("data-")
        .unwrap_or(attribute_name);
    let mut output = String::new();
    let mut uppercase = false;
    for character in source.chars() {
        if character == '-' {
            uppercase = true;
        } else if uppercase {
            output.extend(character.to_uppercase());
            uppercase = false;
        } else {
            output.push(character);
        }
    }
    output
}

fn attribute_name(property_name: &str) -> Result<String, ()> {
    let bytes = property_name.as_bytes();
    if bytes
        .windows(2)
        .any(|pair| pair[0] == b'-' && pair[1].is_ascii_lowercase())
    {
        return Err(());
    }
    let mut output = String::from("data-");
    for character in property_name.chars() {
        if character.is_ascii_uppercase() {
            output.push('-');
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    Ok(output)
}

fn owner<'s>(
    scope: &v8::PinScope<'s, '_>,
    map: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    scope
        .get_slot::<DomStringMapStore>()?
        .owners
        .get(&map.get_identity_hash().get())
        .map(|owner| v8::Local::new(scope, owner))
}

fn property_name(scope: &v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    if key.is_symbol() {
        return None;
    }
    key.to_string(scope)
        .map(|key| key.to_rust_string_lossy(scope))
}

fn named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "get", key, None);
    let (Some(owner), Some(property)) =
        (owner(scope, arguments.holder()), property_name(scope, key))
    else {
        return v8::Intercepted::kNo;
    };
    let Ok(attribute) = attribute_name(&property) else {
        return v8::Intercepted::kNo;
    };
    let Some(value) = super::element::attribute_value(scope, owner, &attribute) else {
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
    let (Some(owner), Some(property)) =
        (owner(scope, arguments.holder()), property_name(scope, key))
    else {
        return v8::Intercepted::kNo;
    };
    let Ok(attribute) = attribute_name(&property) else {
        super::node::throw_dom_exception(
            scope,
            "SyntaxError",
            "The dataset property name contains an invalid hyphen sequence",
        );
        return v8::Intercepted::kYes;
    };
    let value = crate::webidl::value_to_string(scope, value);
    super::element::set_attribute_full(scope, owner, attribute, value, None);
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
    let (Some(owner), Some(property)) =
        (owner(scope, arguments.holder()), property_name(scope, key))
    else {
        return v8::Intercepted::kNo;
    };
    let present = attribute_name(&property)
        .ok()
        .and_then(|attribute| super::element::attribute_value(scope, owner, &attribute))
        .is_some();
    if present {
        result.set_int32(0);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn named_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "delete", key, None);
    let (Some(owner), Some(property)) =
        (owner(scope, arguments.holder()), property_name(scope, key))
    else {
        return v8::Intercepted::kNo;
    };
    let Ok(attribute) = attribute_name(&property) else {
        return v8::Intercepted::kNo;
    };
    super::element::remove_attribute_full(scope, owner, None, &attribute);
    result.set_bool(true);
    v8::Intercepted::kYes
}

fn named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let Some(owner) = owner(scope, arguments.holder()) else {
        result.set(v8::Array::new(scope, 0));
        return;
    };
    let names = super::element::attributes_snapshot(scope, owner)
        .unwrap_or_default()
        .into_iter()
        .filter(|attribute| attribute.name.starts_with("data-"))
        .filter_map(|attribute| {
            v8::String::new(scope, &dataset_name(&attribute.name)).map(|name| name.into())
        })
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &names));
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}
