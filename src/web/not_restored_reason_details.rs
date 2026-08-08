use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NotRestoredReasonDetailsStore {
    constructor: crate::webidl::RealmConstructor,
    reasons: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NotRestoredReasonDetailsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NotRestoredReasonDetails", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<NotRestoredReasonDetailsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NotRestoredReasonDetails",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reason", get_reason)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NotRestoredReasonDetailsStore>()
        .ok_or_else(|| "NotRestoredReasonDetails state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    reason: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let details = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, details, prototype.into()) != Some(true) {
        return Err("cannot create NotRestoredReasonDetails".to_owned());
    }
    scope
        .get_slot_mut::<NotRestoredReasonDetailsStore>()
        .ok_or_else(|| "NotRestoredReasonDetails state was not prepared".to_owned())?
        .reasons
        .insert(details.get_identity_hash().get(), reason);
    Ok(details)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NotRestoredReasonDetails': Illegal constructor",
    );
}

fn reason_for(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<String> {
    scope
        .get_slot::<NotRestoredReasonDetailsStore>()?
        .reasons
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_reason(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(reason) = reason_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &reason) {
        result.set(value.into());
    }
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(reason) = reason_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = v8::Object::new(scope);
    let Ok(key) = crate::webidl::string(scope, "reason") else {
        return;
    };
    let Some(reason) = v8::String::new(scope, &reason) else {
        return;
    };
    if value.define_own_property(
        scope,
        key.into(),
        reason.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        result.set(value.into());
    }
}
