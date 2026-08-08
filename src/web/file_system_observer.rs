use std::collections::HashMap;
#[derive(Clone)]
struct ObserverRecord {
    callback: v8::Global<v8::Function>,
    targets: Vec<v8::Global<v8::Object>>,
}
#[derive(Default)]
pub(crate) struct FileSystemObserverStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ObserverRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(FileSystemObserverStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "FileSystemObserver", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<FileSystemObserverStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "FileSystemObserver",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "disconnect", 0, disconnect)?;
    crate::webidl::define_method(s, p, "observe", 1, observe)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<FileSystemObserverStore>()
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
        crate::webidl::throw_type_error(s, "callback must be callable");
        return;
    };
    let callback = v8::Global::new(s, callback);
    s.get_slot_mut::<FileSystemObserverStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            ObserverRecord {
                callback,
                targets: Vec::new(),
            },
        );
    r.set(a.this().into())
}
fn disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<FileSystemObserverStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.targets.clear()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn observe(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Ok(target) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "target must be a FileSystemHandle");
        return;
    };
    let Some(record) = s
        .get_slot::<FileSystemObserverStore>()
        .and_then(|x| x.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let target = v8::Global::new(s, target);
    if let Some(v) = s
        .get_slot_mut::<FileSystemObserverStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.targets.push(target);
    }
    let callback = v8::Local::new(s, &record.callback);
    let changes = v8::Array::new(s, 0);
    let _ = callback.call(s, a.this().into(), &[changes.into(), a.this().into()]);
    let x = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
        r.set(p.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FileSystemObserverStore>() {
        store.constructor.remove(realm_id);
    }
}
