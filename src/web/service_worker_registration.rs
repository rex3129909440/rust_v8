use std::collections::HashMap;

#[derive(Clone)]
struct Registration {
    logical_id: i32,
    interface_realm_id: i32,
    container_id: i32,
    scope_url: String,
    update_via_cache: String,
    installing: Option<v8::Global<v8::Object>>,
    waiting: Option<v8::Global<v8::Object>>,
    active: Option<v8::Global<v8::Object>>,
    navigation: v8::Global<v8::Object>,
    payment: v8::Global<v8::Object>,
    background_fetch: v8::Global<v8::Object>,
    periodic_sync: v8::Global<v8::Object>,
    sync: v8::Global<v8::Object>,
    cookies: v8::Global<v8::Object>,
    push: v8::Global<v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
    notifications: Vec<v8::Global<v8::Object>>,
    unregistered: bool,
}

#[derive(Default)]
pub(crate) struct ServiceWorkerRegistrationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Registration>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ServiceWorkerRegistrationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "ServiceWorkerRegistration", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<ServiceWorkerRegistrationStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ServiceWorkerRegistration",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::service_worker_registration_installing_property::define(scope, prototype)?;
    super::service_worker_registration_waiting_property::define(scope, prototype)?;
    super::service_worker_registration_active_property::define(scope, prototype)?;
    super::service_worker_registration_navigation_preload_property::define(scope, prototype)?;
    super::service_worker_registration_scope_property::define(scope, prototype)?;
    super::service_worker_registration_update_via_cache_property::define(scope, prototype)?;
    super::service_worker_registration_onupdatefound_property::define(scope, prototype)?;
    super::service_worker_registration_unregister::define(scope, prototype)?;
    super::service_worker_registration_update::define(scope, prototype)?;
    super::service_worker_registration_payment_manager_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::service_worker_registration_background_fetch_property::define(scope, prototype)?;
    super::service_worker_registration_periodic_sync_property::define(scope, prototype)?;
    super::service_worker_registration_sync_property::define(scope, prototype)?;
    super::service_worker_registration_cookies_property::define(scope, prototype)?;
    super::service_worker_registration_push_manager_property::define(scope, prototype)?;
    super::service_worker_registration_get_notifications::define(scope, prototype)?;
    super::service_worker_registration_show_notification::define(scope, prototype)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ServiceWorkerRegistrationStore>()
        .ok_or_else(|| "ServiceWorkerRegistration state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    container_id: i32,
    scope_url: String,
    update_via_cache: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ServiceWorkerRegistration".to_owned());
    }
    super::event_target::attach(scope, object);
    let navigation = super::navigation_preload_manager::create(scope)?;
    let payment = super::payment_manager::create(scope)?;
    let background_fetch = super::background_fetch_manager::create(scope)?;
    let periodic_sync = super::periodic_sync_manager::create(scope)?;
    let sync = super::sync_manager::create(scope)?;
    let cookies = super::cookie_store_manager::create(scope)?;
    let push = super::push_manager::create(scope)?;
    let object_id = object.get_identity_hash().get();
    let record = Registration {
        logical_id: object_id,
        interface_realm_id: crate::webidl::realm_id(scope),
        container_id,
        scope_url,
        update_via_cache,
        installing: None,
        waiting: None,
        active: None,
        navigation: v8::Global::new(scope, navigation),
        payment: v8::Global::new(scope, payment),
        background_fetch: v8::Global::new(scope, background_fetch),
        periodic_sync: v8::Global::new(scope, periodic_sync),
        sync: v8::Global::new(scope, sync),
        cookies: v8::Global::new(scope, cookies),
        push: v8::Global::new(scope, push),
        handler: None,
        notifications: Vec::new(),
        unregistered: false,
    };
    scope
        .get_slot_mut::<ServiceWorkerRegistrationStore>()
        .ok_or_else(|| "ServiceWorkerRegistration state was not prepared".to_owned())?
        .records
        .insert(object_id, record);
    Ok(object)
}

pub(crate) fn create_alias<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
    worker: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let source = record(scope, source)
        .ok_or_else(|| "ServiceWorkerRegistration source wrapper is unavailable".to_owned())?;
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ServiceWorkerRegistration realm wrapper".to_owned());
    }
    super::event_target::attach(scope, object);
    let navigation = super::navigation_preload_manager::create(scope)?;
    let payment = super::payment_manager::create(scope)?;
    let background_fetch = super::background_fetch_manager::create(scope)?;
    let periodic_sync = super::periodic_sync_manager::create(scope)?;
    let sync = super::sync_manager::create(scope)?;
    let cookies = super::cookie_store_manager::create(scope)?;
    let push = super::push_manager::create(scope)?;
    let worker = v8::Global::new(scope, worker);
    let record = Registration {
        logical_id: source.logical_id,
        interface_realm_id: crate::webidl::realm_id(scope),
        container_id: source.container_id,
        scope_url: source.scope_url,
        update_via_cache: source.update_via_cache,
        installing: source.installing.is_some().then_some(worker.clone()),
        waiting: source.waiting.is_some().then_some(worker.clone()),
        active: source.active.is_some().then_some(worker),
        navigation: v8::Global::new(scope, navigation),
        payment: v8::Global::new(scope, payment),
        background_fetch: v8::Global::new(scope, background_fetch),
        periodic_sync: v8::Global::new(scope, periodic_sync),
        sync: v8::Global::new(scope, sync),
        cookies: v8::Global::new(scope, cookies),
        push: v8::Global::new(scope, push),
        handler: None,
        notifications: Vec::new(),
        unregistered: source.unregistered,
    };
    scope
        .get_slot_mut::<ServiceWorkerRegistrationStore>()
        .ok_or_else(|| "ServiceWorkerRegistration state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Registration> {
    scope
        .get_slot::<ServiceWorkerRegistrationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) enum WorkerSlot {
    Installing,
    Waiting,
    Active,
}

pub(crate) fn get_worker(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    slot: WorkerSlot,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = match slot {
        WorkerSlot::Installing => record.installing,
        WorkerSlot::Waiting => record.waiting,
        WorkerSlot::Active => record.active,
    };
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) enum ObjectProperty {
    NavigationPreload,
    PaymentManager,
    BackgroundFetch,
    PeriodicSync,
    Sync,
    Cookies,
    PushManager,
}

pub(crate) fn get_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    property: ObjectProperty,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = match property {
        ObjectProperty::NavigationPreload => record.navigation,
        ObjectProperty::PaymentManager => record.payment,
        ObjectProperty::BackgroundFetch => record.background_fetch,
        ObjectProperty::PeriodicSync => record.periodic_sync,
        ObjectProperty::Sync => record.sync,
        ObjectProperty::Cookies => record.cookies,
        ObjectProperty::PushManager => record.push,
    };
    result.set(v8::Local::new(scope, &value).into());
}

pub(crate) enum TextProperty {
    Scope,
    UpdateViaCache,
}

pub(crate) fn get_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    property: TextProperty,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = match property {
        TextProperty::Scope => record.scope_url,
        TextProperty::UpdateViaCache => record.update_via_cache,
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|record| record.handler);
    super::window_event_handler_support::return_handler(scope, handler, result);
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<ServiceWorkerRegistrationStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.handler = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

pub(crate) fn unregister(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.unregistered {
        resolve(scope, v8::Boolean::new(scope, false).into(), result);
        return;
    }
    let (active, container_id) = {
        let record = scope
            .get_slot_mut::<ServiceWorkerRegistrationStore>()
            .and_then(|store| store.records.get_mut(&id))
            .expect("ServiceWorkerRegistration record");
        record.unregistered = true;
        let active = record.active.take();
        record.installing = None;
        record.waiting = None;
        (active, record.container_id)
    };
    if let Some(active) = active {
        super::service_worker::make_redundant(scope, v8::Local::new(scope, &active));
    }
    super::service_worker_container::remove_registration(scope, container_id, id);
    resolve(scope, v8::Boolean::new(scope, true).into(), result);
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        resolve(scope, arguments.this().into(), result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn show_notification(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let title = crate::webidl::value_to_string(scope, arguments.get(0));
    let notification = {
        let global = scope.get_current_context().global(scope);
        let Some(key) = v8::String::new(scope, "Notification") else {
            return;
        };
        let Some(value) = global.get(scope, key.into()) else {
            return;
        };
        let Ok(constructor) = v8::Local::<v8::Function>::try_from(value) else {
            return;
        };
        let Some(title) = v8::String::new(scope, &title) else {
            return;
        };
        let Some(object) = constructor.new_instance(scope, &[title.into(), arguments.get(1)])
        else {
            return;
        };
        object
    };
    let notification = v8::Global::new(scope, notification);
    let Some(record) = scope
        .get_slot_mut::<ServiceWorkerRegistrationStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.notifications.push(notification);
    resolve(scope, v8::undefined(scope).into(), result);
}

pub(crate) fn get_notifications(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.notifications.len() as i32);
    for (index, notification) in record.notifications.iter().enumerate() {
        let notification = v8::Local::new(scope, notification);
        let _ = array.set_index(scope, index as u32, notification.into());
    }
    resolve(scope, array.into(), result);
}

pub(crate) fn set_installing(
    scope: &mut v8::PinScope<'_, '_>,
    registration: v8::Local<'_, v8::Object>,
    worker: v8::Local<'_, v8::Object>,
) {
    let worker = v8::Global::new(scope, worker);
    let handler = {
        let Some(record) = scope
            .get_slot_mut::<ServiceWorkerRegistrationStore>()
            .and_then(|store| {
                store
                    .records
                    .get_mut(&registration.get_identity_hash().get())
            })
        else {
            return;
        };
        record.installing = Some(worker);
        record.handler.clone()
    };
    let event = super::event_target::create_event(scope, "updatefound");
    super::event_target::dispatch(scope, registration, event);
    if let Some(handler) = handler
        && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = handler.call(scope, registration.into(), &[event.into()]);
    }
}

pub(crate) fn activate(
    scope: &mut v8::PinScope<'_, '_>,
    registration: v8::Local<'_, v8::Object>,
    worker: v8::Local<'_, v8::Object>,
) {
    let Some(logical_id) = record(scope, registration).map(|record| record.logical_id) else {
        return;
    };
    let Some(worker_logical_id) = super::service_worker::logical_id(scope, worker) else {
        return;
    };
    let aliases = scope
        .get_slot::<ServiceWorkerRegistrationStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, record)| record.logical_id == logical_id)
                .map(|(id, record)| (*id, record.interface_realm_id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(id, realm_id)| {
            super::service_worker::wrapper_for_realm(scope, worker_logical_id, realm_id)
                .map(|worker| (id, worker))
        })
        .collect::<Vec<_>>();
    if let Some(store) = scope.get_slot_mut::<ServiceWorkerRegistrationStore>() {
        for (id, worker) in aliases {
            if let Some(record) = store.records.get_mut(&id) {
                record.installing = None;
                record.waiting = None;
                record.active = Some(worker);
            }
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ServiceWorkerRegistrationStore>() {
        store.constructor.remove(realm_id);
    }
}
