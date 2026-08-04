use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct PendingDelivery {
    callback: v8::Global<v8::Function>,
    value: v8::Global<v8::Object>,
}
#[derive(Default)]
pub(crate) struct GeolocationStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
    watches: HashMap<i32, v8::Global<v8::Function>>,
    pending: HashMap<i32, PendingDelivery>,
    next_watch: i32,
    next_delivery: i32,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(GeolocationStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "Geolocation", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<GeolocationStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "Geolocation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "clearWatch", 1, clear_watch)?;
    crate::webidl::define_method(s, p, "getCurrentPosition", 1, get_current_position)?;
    crate::webidl::define_method(s, p, "watchPosition", 1, watch_position)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<GeolocationStore>()
        .ok_or_else(|| "Geolocation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create Geolocation".to_owned());
    }
    s.get_slot_mut::<GeolocationStore>()
        .ok_or_else(|| "Geolocation state was not prepared".to_owned())?
        .objects
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn valid(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<GeolocationStore>()
        .is_some_and(|x| x.objects.contains(&o.get_identity_hash().get()))
}
fn position<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Object>, String> {
    let configured = crate::fingerprint::edge(s).geolocation.clone();
    let coords = super::geolocation_coordinates::create(
        s,
        super::geolocation_coordinates::CoordinatesRecord {
            latitude: configured.latitude,
            longitude: configured.longitude,
            altitude: configured.altitude,
            accuracy: configured.accuracy,
            altitude_accuracy: configured.altitude_accuracy,
            heading: configured.heading,
            speed: configured.speed,
        },
    )?;
    let timestamp = crate::determinism::date_epoch_milliseconds(s);
    super::geolocation_position::create(s, coords, timestamp)
}
fn schedule_value(
    s: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
    value: v8::Local<'_, v8::Object>,
) {
    let callback = v8::Global::new(s, callback);
    let value = v8::Global::new(s, value);
    let delivery = {
        let store = s
            .get_slot_mut::<GeolocationStore>()
            .expect("Geolocation state");
        store.next_delivery += 1;
        let id = store.next_delivery;
        store
            .pending
            .insert(id, PendingDelivery { callback, value });
        id
    };
    let data = v8::Integer::new(s, delivery);
    if let Some(task) = v8::Function::builder(deliver)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(s)
    {
        s.enqueue_microtask(task)
    }
}

fn schedule_success(s: &mut v8::PinScope<'_, '_>, callback: v8::Local<'_, v8::Function>) {
    if let Ok(value) = position(s) {
        schedule_value(s, callback, value);
    }
}

fn schedule_permission_error(
    s: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
    permission: &str,
) {
    let message = if permission == "denied" {
        "User denied Geolocation"
    } else {
        "Geolocation permission has not been granted"
    };
    if let Ok(value) = super::geolocation_position_error::create(s, 1, message.to_owned()) {
        schedule_value(s, callback, value);
    }
}
fn deliver(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(id) = a.data().int32_value(s) else {
        return;
    };
    let pending = s
        .get_slot_mut::<GeolocationStore>()
        .and_then(|x| x.pending.remove(&id));
    if let Some(pending) = pending {
        let callback = v8::Local::new(s, &pending.callback);
        let value = v8::Local::new(s, &pending.value);
        let _ = callback.call(s, v8::undefined(s).into(), &[value.into()]);
    }
}
fn clear_watch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let id = a.get(0).int32_value(s).unwrap_or(0);
    if let Some(store) = s.get_slot_mut::<GeolocationStore>() {
        store.watches.remove(&id);
    }
}
fn get_current_position(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "The success callback must be a function");
        return;
    };
    let permission = crate::fingerprint::edge(s).permissions.geolocation.clone();
    if permission == "granted" {
        schedule_success(s, callback);
    } else if let Ok(error_callback) = v8::Local::<v8::Function>::try_from(a.get(1)) {
        schedule_permission_error(s, error_callback, &permission);
    }
}
fn watch_position(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "The success callback must be a function");
        return;
    };
    let permission = crate::fingerprint::edge(s).permissions.geolocation.clone();
    let watch_callback = if permission == "granted" {
        Some(v8::Global::new(s, callback))
    } else {
        None
    };
    let id = {
        let store = s
            .get_slot_mut::<GeolocationStore>()
            .expect("Geolocation state");
        store.next_watch += 1;
        let id = store.next_watch;
        if let Some(watch_callback) = watch_callback {
            store.watches.insert(id, watch_callback);
        }
        id
    };
    if permission == "granted" {
        schedule_success(s, callback);
    } else if let Ok(error_callback) = v8::Local::<v8::Function>::try_from(a.get(1)) {
        schedule_permission_error(s, error_callback, &permission);
    }
    r.set(v8::Integer::new(s, id).into())
}
