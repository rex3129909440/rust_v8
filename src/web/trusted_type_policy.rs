use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TrustedTypePolicyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PolicyRecord>,
}

#[derive(Clone)]
struct PolicyRecord {
    name: String,
    rules: v8::Global<v8::Object>,
    create_html: Option<v8::Global<v8::Function>>,
    create_script: Option<v8::Global<v8::Function>>,
    create_script_url: Option<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TrustedTypePolicyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TrustedTypePolicy", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TrustedTypePolicyStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TrustedTypePolicy",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_method(scope, prototype, "createHTML", 1, create_html)?;
    crate::webidl::define_method(scope, prototype, "createScript", 1, create_script)?;
    crate::webidl::define_method(scope, prototype, "createScriptURL", 1, create_script_url)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TrustedTypePolicyStore>()
        .ok_or_else(|| "TrustedTypePolicy state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    rules: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let policy = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, policy, prototype.into()) != Some(true) {
        return Err("cannot create TrustedTypePolicy".to_owned());
    }
    let record = PolicyRecord {
        name,
        rules: v8::Global::new(scope, rules),
        create_html: function_property(scope, rules, "createHTML"),
        create_script: function_property(scope, rules, "createScript"),
        create_script_url: function_property(scope, rules, "createScriptURL"),
    };
    scope
        .get_slot_mut::<TrustedTypePolicyStore>()
        .ok_or_else(|| "TrustedTypePolicy state was not prepared".to_owned())?
        .records
        .insert(policy.get_identity_hash().get(), record);
    Ok(policy)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TrustedTypePolicy': Illegal constructor",
    );
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<PolicyRecord> {
    scope
        .get_slot::<TrustedTypePolicyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn function_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Global<v8::Function>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    let function = v8::Local::<v8::Function>::try_from(value).ok()?;
    Some(v8::Global::new(scope, function))
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(name) = v8::String::new(scope, &record.name) {
        result.set(name.into());
    }
}

fn apply_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    select: impl FnOnce(&PolicyRecord) -> Option<v8::Global<v8::Function>>,
) -> Option<String> {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return None;
    };
    let Some(callback) = select(&record) else {
        crate::webidl::throw_type_error(scope, "The policy does not define this conversion");
        return None;
    };
    let callback = v8::Local::new(scope, &callback);
    let receiver = v8::Local::new(scope, &record.rules);
    let supplied = arguments.length().max(1);
    let mut values = Vec::with_capacity(supplied as usize);
    for index in 0..supplied {
        values.push(arguments.get(index));
    }
    let output = callback.call(scope, receiver.into(), &values)?;
    Some(crate::webidl::value_to_string(scope, output))
}

fn create_html(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = apply_rule(scope, &arguments, |record| record.create_html.clone()) else {
        return;
    };
    match super::trusted_html::create(scope, value) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_script(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = apply_rule(scope, &arguments, |record| record.create_script.clone()) else {
        return;
    };
    match super::trusted_script::create(scope, value) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_script_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = apply_rule(scope, &arguments, |record| record.create_script_url.clone())
    else {
        return;
    };
    match super::trusted_script_url::create(scope, value) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TrustedTypePolicyStore>() {
        store.constructor.remove(realm_id);
    }
}
