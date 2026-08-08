use std::collections::HashMap;

#[derive(Clone, Default)]
struct AudioParamMapRecord {
    entries: Vec<(String, v8::Global<v8::Object>)>,
}

#[derive(Default)]
pub(crate) struct AudioParamMapStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioParamMapRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioParamMapStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioParamMap", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioParamMapStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioParamMap",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let entries_key = crate::webidl::string(scope, "entries")?;
    let entries_function = prototype
        .get(scope, entries_key.into())
        .ok_or_else(|| "AudioParamMap.entries is unavailable".to_owned())?;
    if prototype.define_own_property(
        scope,
        v8::Symbol::get_iterator(scope).into(),
        entries_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define AudioParamMap iterator".to_owned());
    }
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioParamMapStore>()
        .ok_or_else(|| "AudioParamMap state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    entries: Vec<(String, v8::Local<'_, v8::Object>)>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create AudioParamMap".to_owned());
    }
    let entries = entries
        .into_iter()
        .map(|(name, value)| (name, v8::Global::new(scope, value)))
        .collect();
    scope
        .get_slot_mut::<AudioParamMapStore>()
        .ok_or_else(|| "AudioParamMap state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AudioParamMapRecord { entries },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioParamMapRecord> {
    scope
        .get_slot::<AudioParamMapStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn entry_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Vec<(String, v8::Global<v8::Object>)> {
    record(scope, object)
        .map(|record| record.entries)
        .unwrap_or_default()
}

fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.entries.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

fn iterator(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    kind: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.entries.len() as i32);
    for (index, (name, value)) in record.entries.iter().enumerate() {
        let item: v8::Local<v8::Value> = if kind == "keys" {
            v8::String::new(scope, name).expect("parameter name").into()
        } else if kind == "values" {
            v8::Local::new(scope, value).into()
        } else {
            let pair = v8::Array::new(scope, 2);
            let name = v8::String::new(scope, name).expect("parameter name");
            let _ = pair.set_index(scope, 0, name.into());
            let _ = pair.set_index(scope, 1, v8::Local::new(scope, value).into());
            pair.into()
        };
        let _ = array.set_index(scope, index as u32, item);
    }
    let symbol = v8::Symbol::get_iterator(scope);
    if let Some(function) = array
        .get(scope, symbol.into())
        .and_then(|v| v8::Local::<v8::Function>::try_from(v).ok())
        && let Some(value) = function.call(scope, array.into(), &[])
    {
        result.set(value)
    }
}
fn entries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a.this(), "entries", r)
}
fn keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a.this(), "keys", r)
}
fn values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    iterator(s, a.this(), "values", r)
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, a.get(0));
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some((_, value)) = record.entries.iter().find(|(key, _)| *key == name) {
        r.set(v8::Local::new(scope, value).into())
    }
}
fn has(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, record.entries.iter().any(|(key, _)| *key == name)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback is required");
        return;
    };
    let receiver = if a.get(1).is_undefined() {
        v8::undefined(scope).into()
    } else {
        a.get(1)
    };
    for (name, value) in record.entries {
        let name = v8::String::new(scope, &name).expect("parameter name");
        let _ = callback.call(
            scope,
            receiver,
            &[
                v8::Local::new(scope, &value).into(),
                name.into(),
                a.this().into(),
            ],
        );
    }
}
