use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) enum IdbKey {
    Number(f64),
    Date(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<IdbKey>),
}

#[derive(Default)]
pub(crate) struct IdbKeyRangeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdbKeyRangeRecord>,
}

#[derive(Clone)]
pub(crate) struct IdbKeyRangeRecord {
    pub lower: Option<IdbKey>,
    pub upper: Option<IdbKey>,
    lower_value: Option<v8::Global<v8::Value>>,
    upper_value: Option<v8::Global<v8::Value>>,
    pub lower_open: bool,
    pub upper_open: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbKeyRangeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBKeyRange", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbKeyRangeStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBKeyRange",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lower", get_lower)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "upper", get_upper)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lowerOpen", get_lower_open)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "upperOpen", get_upper_open)?;
    crate::webidl::define_method(scope, prototype, "includes", 1, includes)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "bound", 2, bound)?;
    crate::webidl::define_method(scope, constructor.into(), "lowerBound", 1, lower_bound)?;
    crate::webidl::define_method(scope, constructor.into(), "only", 1, only)?;
    crate::webidl::define_method(scope, constructor.into(), "upperBound", 1, upper_bound)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbKeyRangeStore>()
        .ok_or_else(|| "IDBKeyRange state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn key_from_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<IdbKey> {
    if value.is_number() {
        let number = value.number_value(scope)?;
        return number.is_finite().then_some(IdbKey::Number(number));
    }
    if value.is_string() || value.is_string_object() {
        return Some(IdbKey::String(crate::webidl::value_to_string(scope, value)));
    }
    if value.is_date() {
        let object = v8::Local::<v8::Object>::try_from(value).ok()?;
        let value_of = object.get(scope, v8::String::new(scope, "valueOf")?.into())?;
        let function = v8::Local::<v8::Function>::try_from(value_of).ok()?;
        let milliseconds = function
            .call(scope, object.into(), &[])?
            .number_value(scope)?;
        return milliseconds
            .is_finite()
            .then_some(IdbKey::Date(milliseconds));
    }
    if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
        let mut bytes = vec![0_u8; buffer.byte_length()];
        let backing = buffer.get_backing_store();
        if let Some(data) = backing.data() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr() as *const u8,
                    bytes.as_mut_ptr(),
                    bytes.len(),
                );
            }
        }
        return Some(IdbKey::Binary(bytes));
    }
    if value.is_array() {
        let array = v8::Local::<v8::Array>::try_from(value).ok()?;
        let mut keys = Vec::with_capacity(array.length() as usize);
        for index in 0..array.length() {
            keys.push(key_from_value(scope, array.get_index(scope, index)?)?);
        }
        return Some(IdbKey::Array(keys));
    }
    None
}

pub(crate) fn compare(left: &IdbKey, right: &IdbKey) -> Ordering {
    let left_rank = rank(left);
    let right_rank = rank(right);
    if left_rank != right_rank {
        return left_rank.cmp(&right_rank);
    }
    match (left, right) {
        (IdbKey::Number(left), IdbKey::Number(right))
        | (IdbKey::Date(left), IdbKey::Date(right)) => {
            left.partial_cmp(right).unwrap_or(Ordering::Equal)
        }
        (IdbKey::String(left), IdbKey::String(right)) => left.cmp(right),
        (IdbKey::Binary(left), IdbKey::Binary(right)) => left.cmp(right),
        (IdbKey::Array(left), IdbKey::Array(right)) => {
            for (a, b) in left.iter().zip(right.iter()) {
                let ordering = compare(a, b);
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            left.len().cmp(&right.len())
        }
        _ => Ordering::Equal,
    }
}

fn rank(key: &IdbKey) -> u8 {
    match key {
        IdbKey::Number(_) => 1,
        IdbKey::Date(_) => 2,
        IdbKey::String(_) => 3,
        IdbKey::Binary(_) => 4,
        IdbKey::Array(_) => 5,
    }
}

pub(crate) fn value_for_key<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    key: &IdbKey,
) -> v8::Local<'s, v8::Value> {
    match key {
        IdbKey::Number(value) => v8::Number::new(scope, *value).into(),
        IdbKey::Date(value) => v8::Date::new(scope, *value)
            .map(Into::into)
            .unwrap_or_else(|| v8::undefined(scope).into()),
        IdbKey::String(value) => v8::String::new(scope, value)
            .map(Into::into)
            .unwrap_or_else(|| v8::undefined(scope).into()),
        IdbKey::Binary(bytes) => {
            let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes.clone()).make_shared();
            v8::ArrayBuffer::with_backing_store(scope, &backing).into()
        }
        IdbKey::Array(values) => {
            let array = v8::Array::new(scope, values.len() as i32);
            for (index, value) in values.iter().enumerate() {
                let value = value_for_key(scope, value);
                let _ = array.set_index(scope, index as u32, value);
            }
            array.into()
        }
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IdbKeyRangeRecord> {
    scope
        .get_slot::<IdbKeyRangeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn matches_query(
    scope: &v8::PinScope<'_, '_>,
    query: v8::Local<'_, v8::Value>,
    key: &IdbKey,
) -> bool {
    if query.is_undefined() || query.is_null() {
        return true;
    }
    if let Ok(object) = v8::Local::<v8::Object>::try_from(query) {
        if let Some(range) = record(scope, object) {
            return contains_key(&range, key);
        }
    }
    key_from_value(scope, query).is_some_and(|query| compare(&query, key) == Ordering::Equal)
}

fn contains_key(record: &IdbKeyRangeRecord, key: &IdbKey) -> bool {
    if let Some(lower) = &record.lower {
        let ordering = compare(key, lower);
        if ordering == Ordering::Less || (ordering == Ordering::Equal && record.lower_open) {
            return false;
        }
    }
    if let Some(upper) = &record.upper {
        let ordering = compare(key, upper);
        if ordering == Ordering::Greater || (ordering == Ordering::Equal && record.upper_open) {
            return false;
        }
    }
    true
}

fn make_range<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    lower: Option<(IdbKey, v8::Local<'_, v8::Value>)>,
    upper: Option<(IdbKey, v8::Local<'_, v8::Value>)>,
    lower_open: bool,
    upper_open: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IDBKeyRange".to_owned());
    }
    let lower_value = lower
        .as_ref()
        .map(|(_, value)| v8::Global::new(scope, *value));
    let upper_value = upper
        .as_ref()
        .map(|(_, value)| v8::Global::new(scope, *value));
    scope
        .get_slot_mut::<IdbKeyRangeStore>()
        .ok_or_else(|| "IDBKeyRange state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IdbKeyRangeRecord {
                lower: lower.map(|(key, _)| key),
                upper: upper.map(|(key, _)| key),
                lower_value,
                upper_value,
                lower_open,
                upper_open,
            },
        );
    Ok(object)
}

fn data_error(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "DataError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn only(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let Some(key) = key_from_value(scope, value) else {
        data_error(scope, "The parameter is not a valid key.");
        return;
    };
    match make_range(
        scope,
        Some((key.clone(), value)),
        Some((key, value)),
        false,
        false,
    ) {
        Ok(range) => result.set(range.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn lower_bound(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let Some(key) = key_from_value(scope, value) else {
        data_error(scope, "The parameter is not a valid key.");
        return;
    };
    let open = arguments.get(1).boolean_value(scope);
    match make_range(scope, Some((key, value)), None, open, true) {
        Ok(range) => result.set(range.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn upper_bound(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let Some(key) = key_from_value(scope, value) else {
        data_error(scope, "The parameter is not a valid key.");
        return;
    };
    let open = arguments.get(1).boolean_value(scope);
    match make_range(scope, None, Some((key, value)), true, open) {
        Ok(range) => result.set(range.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn bound(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let lower_value = arguments.get(0);
    let upper_value = arguments.get(1);
    let Some(lower) = key_from_value(scope, lower_value) else {
        data_error(scope, "The lower parameter is not a valid key.");
        return;
    };
    let Some(upper) = key_from_value(scope, upper_value) else {
        data_error(scope, "The upper parameter is not a valid key.");
        return;
    };
    let lower_open = arguments.get(2).boolean_value(scope);
    let upper_open = arguments.get(3).boolean_value(scope);
    let ordering = compare(&lower, &upper);
    if ordering == Ordering::Greater || (ordering == Ordering::Equal && (lower_open || upper_open))
    {
        data_error(scope, "The lower key is greater than the upper key.");
        return;
    }
    match make_range(
        scope,
        Some((lower, lower_value)),
        Some((upper, upper_value)),
        lower_open,
        upper_open,
    ) {
        Ok(range) => result.set(range.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_bound(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IdbKeyRangeRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match select(&record) {
        Some(value) => result.set(v8::Local::new(scope, &value)),
        None => result.set(v8::undefined(scope).into()),
    }
}

fn get_lower(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bound(s, a, r, |record| record.lower_value.clone())
}
fn get_upper(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bound(s, a, r, |record| record.upper_value.clone())
}
fn get_lower_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.lower_open).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_upper_open(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.upper_open).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn includes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(key) = key_from_value(scope, arguments.get(0)) else {
        data_error(scope, "The parameter is not a valid key.");
        return;
    };
    result.set(v8::Boolean::new(scope, contains_key(&record, &key)).into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbKeyRangeStore>() {
        store.constructor.remove(realm_id);
    }
}
