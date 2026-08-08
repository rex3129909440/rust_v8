use std::collections::HashMap;

#[derive(Clone)]
struct PushSubscriptionOptionsRecord {
    user_visible_only: bool,
    application_server_key: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct PushSubscriptionOptionsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PushSubscriptionOptionsRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PushSubscriptionOptionsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PushSubscriptionOptions", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<PushSubscriptionOptionsStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PushSubscriptionOptions",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "userVisibleOnly",
        get_user_visible_only,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "applicationServerKey",
        get_application_server_key,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PushSubscriptionOptionsStore>()
        .ok_or_else(|| "PushSubscriptionOptions state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_from_init<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    init: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let mut user_visible_only = false;
    let mut application_server_key: v8::Local<v8::Value> = v8::null(scope).into();
    if let Ok(object) = v8::Local::<v8::Object>::try_from(init) {
        if let Some(key) = v8::String::new(scope, "userVisibleOnly") {
            if let Some(value) = object.get(scope, key.into()) {
                if !value.is_undefined() {
                    user_visible_only = value.boolean_value(scope);
                }
            }
        }
        if let Some(key) = v8::String::new(scope, "applicationServerKey") {
            if let Some(value) = object.get(scope, key.into()) {
                if !value.is_undefined() {
                    application_server_key = value;
                }
            }
        }
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create PushSubscriptionOptions".to_owned());
    }
    let record = PushSubscriptionOptionsRecord {
        user_visible_only,
        application_server_key: v8::Global::new(scope, application_server_key),
    };
    scope
        .get_slot_mut::<PushSubscriptionOptionsStore>()
        .ok_or_else(|| "PushSubscriptionOptions state is unavailable".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PushSubscriptionOptionsRecord> {
    scope
        .get_slot::<PushSubscriptionOptionsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'PushSubscriptionOptions': Illegal constructor",
    )
}

fn get_user_visible_only(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.user_visible_only).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_application_server_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.application_server_key));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PushSubscriptionOptionsStore>() {
        store.constructor.remove(realm_id);
    }
}
