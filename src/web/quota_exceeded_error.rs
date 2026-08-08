use std::collections::HashMap;

#[derive(Clone, Copy)]
struct QuotaExceededErrorRecord {
    quota: Option<f64>,
    requested: Option<f64>,
}

#[derive(Default)]
pub(crate) struct QuotaExceededErrorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, QuotaExceededErrorRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(QuotaExceededErrorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "QuotaExceededError", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<QuotaExceededErrorStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "QuotaExceededError",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "quota", get_quota)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "requested", get_requested)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::dom_exception::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<QuotaExceededErrorStore>()
        .ok_or_else(|| "QuotaExceededError state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'QuotaExceededError': Please use the 'new' operator.",
        );
        return;
    }
    let message = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let quota = number_property(scope, init, "quota");
    let requested = number_property(scope, init, "requested");
    super::dom_exception::attach(
        scope,
        arguments.this(),
        "QuotaExceededError".to_owned(),
        message,
        22,
    );
    scope
        .get_slot_mut::<QuotaExceededErrorStore>()
        .expect("QuotaExceededError state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            QuotaExceededErrorRecord { quota, requested },
        );
    result.set(arguments.this().into());
}

fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<f64> {
    let object = object?;
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        value.number_value(scope)
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<QuotaExceededErrorRecord> {
    scope
        .get_slot::<QuotaExceededErrorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

fn get_quota(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.quota {
        Some(value) => result.set(v8::Number::new(scope, value).into()),
        None => result.set(v8::null(scope).into()),
    }
}

fn get_requested(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.requested {
        Some(value) => result.set(v8::Number::new(scope, value).into()),
        None => result.set(v8::null(scope).into()),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<QuotaExceededErrorStore>() {
        store.constructor.remove(realm_id);
    }
}
