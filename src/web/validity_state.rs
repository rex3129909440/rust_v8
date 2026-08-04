use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ValidityStateStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ValidityRecord>,
}

#[derive(Clone, Default)]
pub(crate) struct ValidityRecord {
    pub(crate) value_missing: bool,
    pub(crate) type_mismatch: bool,
    pub(crate) pattern_mismatch: bool,
    pub(crate) too_long: bool,
    pub(crate) too_short: bool,
    pub(crate) range_underflow: bool,
    pub(crate) range_overflow: bool,
    pub(crate) step_mismatch: bool,
    pub(crate) bad_input: bool,
    pub(crate) custom_error: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ValidityStateStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ValidityState", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ValidityStateStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ValidityState",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "valueMissing", get_value_missing)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "typeMismatch", get_type_mismatch)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "patternMismatch",
        get_pattern_mismatch,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "tooLong", get_too_long)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "tooShort", get_too_short)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "rangeUnderflow",
        get_range_underflow,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rangeOverflow", get_range_overflow)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "stepMismatch", get_step_mismatch)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "badInput", get_bad_input)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "customError", get_custom_error)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "valid", get_valid)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ValidityStateStore>()
        .ok_or_else(|| "ValidityState state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: ValidityRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ValidityState".to_owned());
    }
    scope
        .get_slot_mut::<ValidityStateStore>()
        .ok_or_else(|| "ValidityState state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    record: ValidityRecord,
) -> bool {
    if let Some(current) = scope
        .get_slot_mut::<ValidityStateStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        *current = record;
        true
    } else {
        false
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ValidityRecord> {
    scope
        .get_slot::<ValidityStateStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_bool(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ValidityRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_value_missing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.value_missing)
}
fn get_type_mismatch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.type_mismatch)
}
fn get_pattern_mismatch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.pattern_mismatch)
}
fn get_too_long(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.too_long)
}
fn get_too_short(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.too_short)
}
fn get_range_underflow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.range_underflow)
}
fn get_range_overflow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.range_overflow)
}
fn get_step_mismatch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.step_mismatch)
}
fn get_bad_input(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.bad_input)
}
fn get_custom_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| v.custom_error)
}
fn get_valid(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_bool(s, a, r, |v| {
        !(v.value_missing
            || v.type_mismatch
            || v.pattern_mismatch
            || v.too_long
            || v.too_short
            || v.range_underflow
            || v.range_overflow
            || v.step_mismatch
            || v.bad_input
            || v.custom_error)
    })
}
