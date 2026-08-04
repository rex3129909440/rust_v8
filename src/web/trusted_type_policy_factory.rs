use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct TrustedTypePolicyFactoryStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, FactoryRecord>,
}

#[derive(Clone)]
struct FactoryRecord {
    policy_names: HashSet<String>,
    default_policy: Option<v8::Global<v8::Object>>,
    empty_html: v8::Global<v8::Object>,
    empty_script: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TrustedTypePolicyFactoryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TrustedTypePolicyFactory", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TrustedTypePolicyFactoryStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TrustedTypePolicyFactory",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "emptyHTML", get_empty_html)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "emptyScript", get_empty_script)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "defaultPolicy", get_default_policy)?;
    crate::webidl::define_method(scope, prototype, "createPolicy", 1, create_policy)?;
    crate::webidl::define_method(scope, prototype, "getAttributeType", 2, get_attribute_type)?;
    crate::webidl::define_method(scope, prototype, "getPropertyType", 2, get_property_type)?;
    crate::webidl::define_method(scope, prototype, "getTypeMapping", 0, get_type_mapping)?;
    crate::webidl::define_method(scope, prototype, "isHTML", 1, is_html)?;
    crate::webidl::define_method(scope, prototype, "isScript", 1, is_script)?;
    crate::webidl::define_method(scope, prototype, "isScriptURL", 1, is_script_url)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TrustedTypePolicyFactoryStore>()
        .ok_or_else(|| "TrustedTypePolicyFactory state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create TrustedTypePolicyFactory".to_owned());
    }
    let empty_html = super::trusted_html::create(scope, String::new())?;
    let empty_script = super::trusted_script::create(scope, String::new())?;
    let empty_html = v8::Global::new(scope, empty_html);
    let empty_script = v8::Global::new(scope, empty_script);
    scope
        .get_slot_mut::<TrustedTypePolicyFactoryStore>()
        .ok_or_else(|| "TrustedTypePolicyFactory state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            FactoryRecord {
                policy_names: HashSet::new(),
                default_policy: None,
                empty_html,
                empty_script,
            },
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TrustedTypePolicyFactory': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<FactoryRecord> {
    scope
        .get_slot::<TrustedTypePolicyFactoryStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_empty_html(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.empty_html).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_empty_script(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.empty_script).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_default_policy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(policy) = record.default_policy {
        result.set(v8::Local::new(scope, &policy).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn create_policy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let rules = v8::Local::<v8::Object>::try_from(arguments.get(1))
        .unwrap_or_else(|_| v8::Object::new(scope));
    let identity = arguments.this().get_identity_hash().get();
    let duplicate = scope
        .get_slot::<TrustedTypePolicyFactoryStore>()
        .and_then(|store| store.records.get(&identity))
        .is_none_or(|record| record.policy_names.contains(&name));
    if duplicate {
        crate::webidl::throw_type_error(scope, "Policy name already exists");
        return;
    }
    let policy = match super::trusted_type_policy::create(scope, name.clone(), rules) {
        Ok(policy) => policy,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let default_policy = (name == "default").then(|| v8::Global::new(scope, policy));
    if let Some(record) = scope
        .get_slot_mut::<TrustedTypePolicyFactoryStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.policy_names.insert(name.clone());
        if let Some(default_policy) = default_policy {
            record.default_policy = Some(default_policy);
        }
    }
    result.set(policy.into());
}

fn return_type_name(
    scope: &v8::PinScope<'_, '_>,
    value: Option<&str>,
    result: &mut v8::ReturnValue<'_>,
) {
    if let Some(value) = value.and_then(|value| v8::String::new(scope, value)) {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_attribute_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let tag = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    let attribute = crate::webidl::value_to_string(scope, arguments.get(1)).to_ascii_lowercase();
    let kind = match (tag.as_str(), attribute.as_str()) {
        ("iframe", "srcdoc") => Some("TrustedHTML"),
        ("script", "src") => Some("TrustedScriptURL"),
        (_, name) if name.starts_with("on") => Some("TrustedScript"),
        _ => None,
    };
    return_type_name(scope, kind, &mut result);
}

fn get_property_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let tag = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_lowercase();
    let property = crate::webidl::value_to_string(scope, arguments.get(1));
    let kind = match (tag.as_str(), property.as_str()) {
        (_, "innerHTML" | "outerHTML") => Some("TrustedHTML"),
        ("script", "src") => Some("TrustedScriptURL"),
        ("script", "text" | "textContent" | "innerText") => Some("TrustedScript"),
        _ => None,
    };
    return_type_name(scope, kind, &mut result);
}

fn get_type_mapping(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mapping = v8::Object::new(scope);
    define_string(scope, mapping, "innerHTML", "TrustedHTML");
    define_string(scope, mapping, "outerHTML", "TrustedHTML");
    define_string(scope, mapping, "script.src", "TrustedScriptURL");
    define_string(scope, mapping, "iframe.srcdoc", "TrustedHTML");
    result.set(mapping.into());
}

fn return_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    valid_receiver: bool,
    value: bool,
    mut result: v8::ReturnValue<'_>,
) {
    if valid_receiver {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn is_html(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(
        scope,
        record(scope, arguments.this()).is_some(),
        super::trusted_html::is_instance(scope, arguments.get(0)),
        result,
    );
}

fn is_script(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(
        scope,
        record(scope, arguments.this()).is_some(),
        super::trusted_script::is_instance(scope, arguments.get(0)),
        result,
    );
}

fn is_script_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_boolean(
        scope,
        record(scope, arguments.this()).is_some(),
        super::trusted_script_url::is_instance(scope, arguments.get(0)),
        result,
    );
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let Some(value) = v8::String::new(scope, value) else {
        return;
    };
    let _ = object.create_data_property(scope, key.into(), value.into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TrustedTypePolicyFactoryStore>() {
        store.constructors.remove(&realm_id);
    }
}
