use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct CookieDeprecationLabelStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CookieDeprecationLabelStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "CookieDeprecationLabel", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<CookieDeprecationLabelStore>()
        .and_then(|s| s.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CookieDeprecationLabel",
        0,
        v8::ConstructorBehavior::Allow,
        super::android_api_support::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getValue", 0, get_value)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::android_api_support::set_tag(scope, prototype, "CookieDeprecationLabel")?;
    let stored_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CookieDeprecationLabelStore>()
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
        return Err("cannot create CookieDeprecationLabel".to_owned());
    }
    scope
        .get_slot_mut::<CookieDeprecationLabelStore>()
        .unwrap()
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let valid = scope
        .get_slot::<CookieDeprecationLabelStore>()
        .expect("CookieDeprecationLabel state")
        .instances
        .contains(&arguments.this().get_identity_hash().get());
    if !super::android_api_support::require_brand(
        scope,
        valid,
        "CookieDeprecationLabel",
        "getValue",
    ) {
        return;
    }
    let value = v8::String::new(scope, "").expect("empty string");
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}
