use std::collections::HashMap;
#[derive(Clone)]
struct ObserverData {
    callback: v8::Global<v8::Function>,
    sources: Vec<String>,
    records: Vec<v8::Global<v8::Object>>,
}
#[derive(Default)]
pub(crate) struct PressureObserverStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ObserverData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PressureObserverStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PressureObserver", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<PressureObserverStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PressureObserver",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "disconnect", 0, disconnect)?;
    crate::webidl::define_method(s, p, "observe", 1, observe)?;
    crate::webidl::define_method(s, p, "takeRecords", 0, take_records)?;
    crate::webidl::define_method(s, p, "unobserve", 1, unobserve)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let key = v8::String::new(s, "knownSources").unwrap();
    let array = v8::Array::new(s, 1);
    let value = v8::String::new(s, "cpu").unwrap();
    let _ = array.set_index(s, 0, value.into());
    let _ = c.define_own_property(
        s,
        key.into(),
        array.into(),
        v8::PropertyAttribute::READ_ONLY,
    );
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PressureObserverStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "constructor must be called with new");
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "callback required");
        return;
    };
    let callback = v8::Global::new(s, callback);
    s.get_slot_mut::<PressureObserverStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            ObserverData {
                callback,
                sources: Vec::new(),
                records: Vec::new(),
            },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ObserverData> {
    s.get_slot::<PressureObserverStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<PressureObserverStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.sources.clear();
        v.records.clear()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn observe(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let source = crate::webidl::value_to_string(s, a.get(0));
    let Some(snapshot) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let item = match super::pressure_record::create(s, source.clone(), "nominal".to_owned(), 0.0) {
        Ok(v) => v,
        Err(e) => {
            crate::webidl::throw_type_error(s, &e);
            return;
        }
    };
    let stored = v8::Global::new(s, item);
    if let Some(v) = s
        .get_slot_mut::<PressureObserverStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.sources.push(source);
        v.records.push(stored);
    }
    let callback = v8::Local::new(s, &snapshot.callback);
    let array = v8::Array::new(s, 1);
    let _ = array.set_index(s, 0, item.into());
    let _ = callback.call(s, a.this().into(), &[array.into(), a.this().into()]);
    let x = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
        r.set(p.into())
    }
}
fn take_records(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, v.records.len() as i32);
    for (i, item) in v.records.iter().enumerate() {
        let x = v8::Local::new(s, item);
        let _ = array.set_index(s, i as u32, x.into());
    }
    if let Some(v) = s
        .get_slot_mut::<PressureObserverStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.records.clear();
    }
    r.set(array.into())
}
fn unobserve(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let source = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<PressureObserverStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.sources.retain(|x| x != &source)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PressureObserverStore>() {
        store.constructor.remove(realm_id);
    }
}
