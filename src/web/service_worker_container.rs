use std::collections::HashMap;

#[derive(Clone)]
struct ContainerRecord {
    object: v8::Global<v8::Object>,
    parent_context: v8::Global<v8::Context>,
    controller: Option<v8::Global<v8::Object>>,
    registrations: Vec<v8::Global<v8::Object>>,
    on_controller: Option<v8::Global<v8::Value>>,
    on_message: Option<v8::Global<v8::Value>>,
    on_message_error: Option<v8::Global<v8::Value>>,
    pending_messages: Vec<super::worker_structured_clone::SerializedMessage>,
    ready_waiters: Vec<v8::Global<v8::PromiseResolver>>,
    messages_started: bool,
}

#[derive(Default)]
pub(crate) struct ServiceWorkerContainerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ContainerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ServiceWorkerContainerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "ServiceWorkerContainer", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<ServiceWorkerContainerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ServiceWorkerContainer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::service_worker_container_controller_property::define(scope, prototype)?;
    super::service_worker_container_ready_property::define(scope, prototype)?;
    super::service_worker_container_oncontrollerchange_property::define(scope, prototype)?;
    super::service_worker_container_onmessage_property::define(scope, prototype)?;
    super::service_worker_container_onmessageerror_property::define(scope, prototype)?;
    super::service_worker_container_get_registration::define(scope, prototype)?;
    super::service_worker_container_get_registrations::define(scope, prototype)?;
    super::service_worker_container_register::define(scope, prototype)?;
    super::service_worker_container_start_messages::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ServiceWorkerContainerStore>()
        .ok_or_else(|| "ServiceWorkerContainer state was not prepared".to_owned())?
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
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ServiceWorkerContainer".to_owned());
    }
    super::event_target::attach(scope, object);
    let record = ContainerRecord {
        object: v8::Global::new(scope, object),
        parent_context: v8::Global::new(scope, scope.get_current_context()),
        controller: None,
        registrations: Vec::new(),
        on_controller: None,
        on_message: None,
        on_message_error: None,
        pending_messages: Vec::new(),
        ready_waiters: Vec::new(),
        messages_started: false,
    };
    scope
        .get_slot_mut::<ServiceWorkerContainerStore>()
        .ok_or_else(|| "ServiceWorkerContainer state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ContainerRecord> {
    scope
        .get_slot::<ServiceWorkerContainerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_controller(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(controller) = record.controller {
        result.set(v8::Local::new(scope, &controller).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn resolved(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

fn rejected(scope: &mut v8::PinScope<'_, '_>, message: &str, mut result: v8::ReturnValue<'_>) {
    let exception = v8::Exception::type_error(
        scope,
        v8::String::new(scope, message).unwrap_or_else(|| v8::String::empty(scope)),
    );
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
        result.set(promise.into());
    }
}

pub(crate) fn get_ready(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(registration) = record.registrations.first() {
        let registration = v8::Local::new(scope, registration);
        resolved(scope, registration.into(), result);
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    result.set(resolver.get_promise(scope).into());
    let resolver = v8::Global::new(scope, resolver);
    if let Some(record) = scope
        .get_slot_mut::<ServiceWorkerContainerStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.ready_waiters.push(resolver);
    }
}

pub(crate) fn get_registration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = record
        .registrations
        .first()
        .map(|registration| v8::Local::new(scope, registration).into())
        .unwrap_or_else(|| v8::undefined(scope).into());
    resolved(scope, value, result);
}

pub(crate) fn get_registrations(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.registrations.len() as i32);
    for (index, registration) in record.registrations.iter().enumerate() {
        let registration = v8::Local::new(scope, registration);
        let _ = array.set_index(scope, index as u32, registration.into());
    }
    resolved(scope, array.into(), result);
}

pub(crate) fn register(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "Failed to execute 'register': 1 argument required");
        return;
    }
    let container = arguments.this();
    let container_id = container.get_identity_hash().get();
    if record(scope, container).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let worker_type = option_string(scope, options, "type").unwrap_or_else(|| "classic".to_owned());
    if worker_type != "classic" && worker_type != "module" {
        rejected(
            scope,
            "ServiceWorker type must be 'classic' or 'module'",
            result,
        );
        return;
    }
    let update_via_cache =
        option_string(scope, options, "updateViaCache").unwrap_or_else(|| "imports".to_owned());
    if update_via_cache != "imports" && update_via_cache != "all" && update_via_cache != "none" {
        rejected(scope, "Invalid updateViaCache value", result);
        return;
    }
    let script = match super::worker_script_source::load(scope, &input, None) {
        Ok(script) => script,
        Err(message) => {
            rejected(scope, &message, result);
            return;
        }
    };
    let scope_url = option_string(scope, options, "scope")
        .unwrap_or_else(|| "https://sandbox.test/".to_owned());
    let registration = match super::service_worker_registration::create(
        scope,
        container_id,
        scope_url,
        update_via_cache,
    ) {
        Ok(registration) => registration,
        Err(message) => {
            rejected(scope, &message, result);
            return;
        }
    };
    let worker = match super::service_worker::create(scope, script.url.clone(), container_id) {
        Ok(worker) => worker,
        Err(message) => {
            rejected(scope, &message, result);
            return;
        }
    };
    super::service_worker_registration::set_installing(scope, registration, worker);
    let realm_id = match super::worker_global_scope::create(
        scope,
        super::worker_global_scope::RealmOwner::Service(container_id),
        super::worker_global_scope::RealmKind::Service,
        script.url,
        String::new(),
        worker_type == "module",
    ) {
        Ok(realm_id) => realm_id,
        Err(message) => {
            rejected(scope, &message, result);
            return;
        }
    };
    if let Err(message) = super::worker_global_scope::bind_service_objects(
        scope,
        realm_id,
        registration,
        worker,
        container_id,
    ) {
        super::service_worker::make_redundant(scope, worker);
        rejected(scope, &message, result);
        return;
    }
    if let Err(error) = super::worker_global_scope::evaluate(scope, realm_id, &script.source) {
        super::service_worker::make_redundant(scope, worker);
        super::service_worker::dispatch_error(scope, worker, error.message.clone());
        rejected(scope, &error.message, result);
        return;
    }
    super::worker_global_scope::dispatch_service_lifecycle(
        scope,
        realm_id,
        "install",
        super::worker_global_scope::ServiceHandlerKind::Install,
    );
    super::service_worker::activate(scope, worker, realm_id);
    super::service_worker_registration::activate(scope, registration, worker);
    super::worker_global_scope::dispatch_service_lifecycle(
        scope,
        realm_id,
        "activate",
        super::worker_global_scope::ServiceHandlerKind::Activate,
    );
    finish_registration(scope, container, registration, worker);
    resolved(scope, registration.into(), result);
}

fn option_string(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    options?
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
}

fn finish_registration(
    scope: &mut v8::PinScope<'_, '_>,
    container: v8::Local<'_, v8::Object>,
    registration: v8::Local<'_, v8::Object>,
    worker: v8::Local<'_, v8::Object>,
) {
    let id = container.get_identity_hash().get();
    let worker_global = v8::Global::new(scope, worker);
    let registration_global = v8::Global::new(scope, registration);
    let (handler, waiters) = {
        let Some(record) = scope
            .get_slot_mut::<ServiceWorkerContainerStore>()
            .and_then(|store| store.records.get_mut(&id))
        else {
            return;
        };
        record.controller = Some(worker_global);
        record.registrations.push(registration_global);
        (
            record.on_controller.clone(),
            std::mem::take(&mut record.ready_waiters),
        )
    };
    for waiter in waiters {
        let waiter = v8::Local::new(scope, &waiter);
        let _ = waiter.resolve(scope, registration.into());
    }
    let event = super::event_target::create_event(scope, "controllerchange");
    super::event_target::dispatch(scope, container, event);
    if let Some(handler) = handler
        && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = handler.call(scope, container.into(), &[event.into()]);
    }
}

#[derive(Clone, Copy)]
pub(crate) enum HandlerKind {
    ControllerChange,
    Message,
    MessageError,
}

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    kind: HandlerKind,
    result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|record| match kind {
        HandlerKind::ControllerChange => record.on_controller,
        HandlerKind::Message => record.on_message,
        HandlerKind::MessageError => record.on_message_error,
    });
    super::window_event_handler_support::return_handler(scope, handler, result);
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    kind: HandlerKind,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<ServiceWorkerContainerStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match kind {
        HandlerKind::ControllerChange => record.on_controller = handler,
        HandlerKind::Message => {
            record.on_message = handler;
            record.messages_started = true;
        }
        HandlerKind::MessageError => record.on_message_error = handler,
    }
}

pub(crate) fn start_messages(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<ServiceWorkerContainerStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.messages_started = true;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn queue_from_service(
    scope: &mut v8::PinScope<'_, '_>,
    container_id: i32,
    message: super::worker_structured_clone::SerializedMessage,
) {
    if let Some(record) = scope
        .get_slot_mut::<ServiceWorkerContainerStore>()
        .and_then(|store| store.records.get_mut(&container_id))
    {
        record.pending_messages.push(message);
    }
}

pub(crate) fn remove_registration(
    scope: &mut v8::PinScope<'_, '_>,
    container_id: i32,
    registration_id: i32,
) {
    let registrations = scope
        .get_slot::<ServiceWorkerContainerStore>()
        .and_then(|store| store.records.get(&container_id))
        .map(|record| {
            record
                .registrations
                .iter()
                .filter(|registration| {
                    v8::Local::new(scope, *registration)
                        .get_identity_hash()
                        .get()
                        != registration_id
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(record) = scope
        .get_slot_mut::<ServiceWorkerContainerStore>()
        .and_then(|store| store.records.get_mut(&container_id))
    {
        record.registrations = registrations;
        record.controller = None;
    }
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) {
    let ids = scope
        .get_slot::<ServiceWorkerContainerStore>()
        .map(|store| store.records.keys().copied().collect::<Vec<_>>())
        .unwrap_or_default();
    for id in ids {
        let Some(record) = scope
            .get_slot::<ServiceWorkerContainerStore>()
            .and_then(|store| store.records.get(&id))
            .cloned()
        else {
            continue;
        };
        if !record.messages_started || record.pending_messages.is_empty() {
            continue;
        }
        let messages = scope
            .get_slot_mut::<ServiceWorkerContainerStore>()
            .and_then(|store| store.records.get_mut(&id))
            .map(|record| std::mem::take(&mut record.pending_messages))
            .unwrap_or_default();
        let context = v8::Local::new(scope, &record.parent_context);
        let parent_scope = &mut v8::ContextScope::new(scope, context);
        let target = v8::Local::new(parent_scope, &record.object);
        for message in messages {
            let Some(data) = super::worker_structured_clone::deserialize(parent_scope, &message)
            else {
                dispatch_container_event(
                    parent_scope,
                    target,
                    "messageerror",
                    record.on_message_error.clone(),
                    None,
                );
                continue;
            };
            dispatch_container_event(
                parent_scope,
                target,
                "message",
                record.on_message.clone(),
                Some(data),
            );
        }
    }
}

pub(crate) fn active_realm_id(scope: &v8::PinScope<'_, '_>) -> Option<i32> {
    let controller = scope
        .get_slot::<ServiceWorkerContainerStore>()?
        .records
        .values()
        .find_map(|record| record.controller.as_ref())?
        .clone();
    let controller = v8::Local::new(scope, &controller);
    super::service_worker::realm_id(scope, controller)
}

fn dispatch_container_event(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event_type: &str,
    handler: Option<v8::Global<v8::Value>>,
    data: Option<v8::Local<'_, v8::Value>>,
) {
    let event = if let Some(data) = data {
        super::message_event::create(scope, event_type, data, "", None, Vec::new()).ok()
    } else {
        Some(super::event_target::create_event(scope, event_type))
    };
    let Some(event) = event else {
        return;
    };
    super::event_target::dispatch(scope, target, event);
    if let Some(handler) = handler
        && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
}
