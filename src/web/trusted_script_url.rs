use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TrustedScriptUrlStore {
    constructor: crate::webidl::RealmConstructor,
    values: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TrustedScriptUrlStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TrustedScriptURL", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TrustedScriptUrlStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TrustedScriptURL",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, return_value)?;
    crate::webidl::define_method(scope, prototype, "toString", 0, return_value)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TrustedScriptUrlStore>()
        .ok_or_else(|| "TrustedScriptURL state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create TrustedScriptURL".to_owned());
    }
    scope
        .get_slot_mut::<TrustedScriptUrlStore>()
        .ok_or_else(|| "TrustedScriptURL state was not prepared".to_owned())?
        .values
        .insert(object.get_identity_hash().get(), value);
    Ok(object)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> bool {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return false;
    };
    scope
        .get_slot::<TrustedScriptUrlStore>()
        .is_some_and(|store| store.values.contains_key(&object.get_identity_hash().get()))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TrustedScriptURL': Illegal constructor",
    );
}

fn return_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope.get_slot::<TrustedScriptUrlStore>().and_then(|store| {
        store
            .values
            .get(&arguments.this().get_identity_hash().get())
            .cloned()
    });
    let Some(value) = value else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TrustedScriptUrlStore>() {
        store.constructor.remove(realm_id);
    }
}
