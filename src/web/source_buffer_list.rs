use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SourceBufferListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
struct Record {
    values: Vec<v8::Global<v8::Object>>,
    onadd: Option<v8::Global<v8::Value>>,
    onremove: Option<v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SourceBufferListStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SourceBufferList", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<SourceBufferListStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "SourceBufferList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "length", get_length)?;
    crate::webidl::define_accessor(scope, p, "onaddsourcebuffer", get_onadd, set_onadd)?;
    crate::webidl::define_accessor(scope, p, "onremovesourcebuffer", get_onremove, set_onremove)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_indexed_iterator(scope, p)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SourceBufferListStore>()
        .ok_or_else(|| "SourceBufferList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create SourceBufferList".to_owned());
    }
    super::event_target::attach(scope, o);
    scope
        .get_slot_mut::<SourceBufferListStore>()
        .ok_or_else(|| "SourceBufferList state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            Record {
                values: Vec::new(),
                onadd: None,
                onremove: None,
            },
        );
    Ok(o)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SourceBufferList': Illegal constructor",
    );
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<SourceBufferListStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(v) = scope
        .get_slot_mut::<SourceBufferListStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, v.values.len() as u32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn handler(
    scope: &v8::PinScope<'_, '_>,
    v: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    if v.is_null() || v.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, v))
    }
}
fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> Option<v8::Global<v8::Value>>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = select(&v) {
        r.set(v8::Local::new(scope, &v))
    } else {
        r.set(v8::null(scope).into())
    }
}
fn get_onadd(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |v| v.onadd.clone())
}
fn get_onremove(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |v| v.onremove.clone())
}
fn set_onadd(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = handler(s, a.get(0));
    update(s, a.this(), |v| v.onadd = h)
}
fn set_onremove(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = handler(s, a.get(0));
    update(s, a.this(), |v| v.onremove = h)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SourceBufferListStore>() {
        store.constructor.remove(realm_id);
    }
}
