use std::collections::{HashMap, HashSet};
#[derive(Default)]
pub(crate) struct PushManagerStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
    subscriptions: HashMap<i32, v8::Global<v8::Object>>,
    next: u64,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PushManagerStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PushManager", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(c) = s
        .get_slot::<PushManagerStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "PushManager",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "getSubscription", 0, get_subscription)?;
    crate::webidl::define_method(s, p, "permissionState", 0, permission_state)?;
    crate::webidl::define_method(s, p, "subscribe", 0, subscribe)?;
    crate::webidl::finish_constructor(s, p, c)?;
    define_static(s, c.into())?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PushManagerStore>()
        .ok_or_else(|| "PushManager state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
#[allow(dead_code)]
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create PushManager".to_owned());
    }
    s.get_slot_mut::<PushManagerStore>()
        .ok_or_else(|| "state missing".to_owned())?
        .objects
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<PushManagerStore>()
        .is_some_and(|x| x.objects.contains(&o.get_identity_hash().get()))
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Failed to construct 'PushManager': Illegal constructor")
}
fn get_subscription(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let v = s
        .get_slot::<PushManagerStore>()
        .and_then(|x| x.subscriptions.get(&a.this().get_identity_hash().get()))
        .cloned();
    let value = v
        .map(|x| v8::Local::new(s, &x).into())
        .unwrap_or_else(|| v8::null(s).into());
    if let Ok(p) = super::writable_stream::resolved_promise(s, value) {
        r.set(p.into())
    }
}
fn permission_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let state = crate::fingerprint::edge(s)
        .permissions
        .notifications
        .clone();
    let Some(v) = v8::String::new(s, &state) else {
        return;
    };
    if let Ok(p) = super::writable_stream::resolved_promise(s, v.into()) {
        r.set(p.into())
    }
}
fn subscribe(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let id = a.this().get_identity_hash().get();
    if let Some(existing) = s
        .get_slot::<PushManagerStore>()
        .and_then(|x| x.subscriptions.get(&id))
        .cloned()
    {
        let v = v8::Local::new(s, &existing);
        if let Ok(p) = super::writable_stream::resolved_promise(s, v.into()) {
            r.set(p.into())
        }
        return;
    }
    let Ok(options) = super::push_subscription_options::create_from_init(s, a.get(0)) else {
        return;
    };
    let next = {
        let x = s.get_slot_mut::<PushManagerStore>().expect("state");
        x.next += 1;
        x.next
    };
    let endpoint = format!("https://push.invalid/subscription/{next}");
    let Ok(sub) = super::push_subscription::create(s, endpoint, options) else {
        return;
    };
    let global = v8::Global::new(s, sub);
    s.get_slot_mut::<PushManagerStore>()
        .expect("state")
        .subscriptions
        .insert(id, global);
    if let Ok(p) = super::writable_stream::resolved_promise(s, sub.into()) {
        r.set(p.into())
    }
}
fn define_static(s: &mut v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Result<(), String> {
    let g = crate::webidl::create_function(
        s,
        "get supportedContentEncodings",
        0,
        v8::ConstructorBehavior::Throw,
        get_encodings,
    )?;
    let u = v8::undefined(s);
    let mut d = v8::PropertyDescriptor::new_from_get_set(g.into(), u.into());
    d.set_enumerable(true);
    d.set_configurable(true);
    let k = crate::webidl::string(s, "supportedContentEncodings")?;
    if o.define_property(s, k.into(), &d) == Some(true) {
        Ok(())
    } else {
        Err("cannot define PushManager.supportedContentEncodings".to_owned())
    }
}
fn get_encodings(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let a = v8::Array::new(s, 2);
    if let Some(v) = v8::String::new(s, "aes128gcm") {
        let _ = a.set_index(s, 0, v.into());
    }
    if let Some(v) = v8::String::new(s, "aesgcm") {
        let _ = a.set_index(s, 1, v.into());
    }
    r.set(a.into())
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PushManagerStore>() {
        store.constructor.remove(realm_id);
    }
}
