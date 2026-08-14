use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct ContactsManagerStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ContactsManagerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "ContactsManager", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<ContactsManagerStore>()
        .and_then(|s| s.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ContactsManager",
        0,
        v8::ConstructorBehavior::Allow,
        super::android_api_support::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getProperties", 0, get_properties)?;
    crate::webidl::define_method(scope, prototype, "select", 1, select)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::android_api_support::set_tag(scope, prototype, "ContactsManager")?;
    let stored_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ContactsManagerStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ContactsManager".to_owned());
    }
    scope
        .get_slot_mut::<ContactsManagerStore>()
        .unwrap()
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn valid(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    operation: &str,
) -> bool {
    let valid = scope
        .get_slot::<ContactsManagerStore>()
        .expect("ContactsManager state")
        .instances
        .contains(&arguments.this().get_identity_hash().get());
    super::android_api_support::require_brand(scope, valid, "ContactsManager", operation)
}

fn get_properties(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, &arguments, "getProperties") {
        return;
    }
    let names = ["address", "email", "icon", "name", "tel"];
    let values = v8::Array::new(scope, names.len() as i32);
    for (index, name) in names.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, name) {
            let _ = values.set_index(scope, index as u32, value.into());
        }
    }
    let _ = values.set_integrity_level(scope, v8::IntegrityLevel::Frozen);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, values.into()) {
        result.set(promise.into());
    }
}

fn select(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, &arguments, "select") {
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'select' on 'ContactsManager': 1 argument required, but only 0 present.",
        );
        return;
    }
    if let Some(promise) = super::android_api_support::rejected_dom_exception(
        scope,
        "SecurityError",
        "Failed to execute 'select' on 'ContactsManager': A user gesture is required to call this method",
    ) {
        result.set(promise.into());
    }
}
