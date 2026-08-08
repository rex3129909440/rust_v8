use std::collections::HashMap;

#[derive(Clone, Default)]
struct TimelineTriggerRangeListRecord {
    ranges: Vec<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct TimelineTriggerRangeListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TimelineTriggerRangeListRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TimelineTriggerRangeListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TimelineTriggerRangeList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<TimelineTriggerRangeListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TimelineTriggerRangeList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "values")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TimelineTriggerRangeListStore>()
        .ok_or_else(|| "TimelineTriggerRangeList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'TimelineTriggerRangeList': Illegal constructor",
    )
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    ranges: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create TimelineTriggerRangeList".to_owned());
    }
    let mut globals = Vec::with_capacity(ranges.len());
    for (index, range) in ranges.into_iter().enumerate() {
        globals.push(v8::Global::new(scope, range));
        if let Some(key) = v8::String::new(scope, &index.to_string()) {
            let _ = object.define_own_property(
                scope,
                key.into(),
                range.into(),
                v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
            );
        }
    }
    scope
        .get_slot_mut::<TimelineTriggerRangeListStore>()
        .ok_or_else(|| "TimelineTriggerRangeList state is unavailable".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            TimelineTriggerRangeListRecord { ranges: globals },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TimelineTriggerRangeListRecord> {
    scope
        .get_slot::<TimelineTriggerRangeListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: &TimelineTriggerRangeListRecord,
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, record.ranges.len() as i32);
    for (index, range) in record.ranges.iter().enumerate() {
        let range = v8::Local::new(scope, range);
        let _ = array.set_index(scope, index as u32, range.into());
    }
    array
}

fn iterator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    method: &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = array(scope, &record);
    let Some(key) = v8::String::new(scope, method) else {
        return;
    };
    let Some(value) = array.get(scope, key.into()) else {
        return;
    };
    let Ok(function) = v8::Local::<v8::Function>::try_from(value) else {
        return;
    };
    if let Some(value) = function.call(scope, array.into(), &[]) {
        result.set(value);
    }
}
fn entries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a, r, "entries")
}
fn keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a, r, "keys")
}
fn values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a, r, "values")
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let this_arg = arguments.get(1);
    for (index, range) in record.ranges.iter().enumerate() {
        let range = v8::Local::new(scope, range);
        let index = v8::Integer::new_from_unsigned(scope, index as u32);
        let _ = callback.call(
            scope,
            this_arg,
            &[range.into(), index.into(), arguments.this().into()],
        );
    }
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.ranges.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).integer_value(scope).unwrap_or(-1);
    if index < 0 {
        result.set(v8::null(scope).into());
        return;
    }
    match record.ranges.get(index as usize) {
        Some(range) => result.set(v8::Local::new(scope, range).into()),
        None => result.set(v8::null(scope).into()),
    }
}
