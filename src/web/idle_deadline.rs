use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct IdleDeadlineStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdleDeadlineRecord>,
}

#[derive(Clone)]
struct IdleDeadlineRecord {
    did_timeout: bool,
    expires_at: Instant,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdleDeadlineStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IdleDeadline", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<IdleDeadlineStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IdleDeadline",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "didTimeout", get_did_timeout)?;
    crate::webidl::define_method(scope, prototype, "timeRemaining", 0, time_remaining)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdleDeadlineStore>()
        .ok_or_else(|| "IdleDeadline state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    did_timeout: bool,
    budget_milliseconds: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IdleDeadline".to_owned());
    }
    let budget = if budget_milliseconds.is_finite() {
        budget_milliseconds.clamp(0.0, 50.0)
    } else {
        0.0
    };
    let expires_at = Instant::now() + Duration::from_secs_f64(budget / 1000.0);
    scope
        .get_slot_mut::<IdleDeadlineStore>()
        .ok_or_else(|| "IdleDeadline state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdleDeadlineRecord {
                did_timeout,
                expires_at,
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
        "Failed to construct 'IdleDeadline': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdleDeadlineRecord> {
    scope
        .get_slot::<IdleDeadlineStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_did_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.did_timeout).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn time_remaining(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let now = Instant::now();
    let remaining = if record.expires_at > now {
        (record.expires_at - now).as_secs_f64() * 1000.0
    } else {
        0.0
    };
    result.set(v8::Number::new(scope, remaining.min(50.0)).into());
}
