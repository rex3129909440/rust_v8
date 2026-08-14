use std::collections::HashMap;

#[derive(Clone)]
struct WorkerRecord {
    object: v8::Global<v8::Object>,
    context: v8::Global<v8::Context>,
    logical_id: i32,
    interface_realm_id: i32,
    url: String,
    state: String,
    container_id: i32,
    worker_realm_id: Option<i32>,
    on_state: Option<v8::Global<v8::Value>>,
    on_error: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct ServiceWorkerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WorkerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ServiceWorkerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "ServiceWorker", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<ServiceWorkerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ServiceWorker",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::service_worker_script_url_property::define(scope, prototype)?;
    super::service_worker_state_property::define(scope, prototype)?;
    super::service_worker_onstatechange_property::define(scope, prototype)?;
    super::service_worker_post_message::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::service_worker_onerror_property::define(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ServiceWorkerStore>()
        .ok_or_else(|| "ServiceWorker state was not prepared".to_owned())?
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
    url: String,
    container_id: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ServiceWorker".to_owned());
    }
    super::event_target::attach(scope, object);
    let object_id = object.get_identity_hash().get();
    let record = WorkerRecord {
        object: v8::Global::new(scope, object),
        context: v8::Global::new(scope, scope.get_current_context()),
        logical_id: object_id,
        interface_realm_id: crate::webidl::realm_id(scope),
        url,
        state: "installing".to_owned(),
        container_id,
        worker_realm_id: None,
        on_state: None,
        on_error: None,
    };
    scope
        .get_slot_mut::<ServiceWorkerStore>()
        .ok_or_else(|| "ServiceWorker state was not prepared".to_owned())?
        .records
        .insert(object_id, record);
    Ok(object)
}

pub(crate) fn create_alias<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let source = record(scope, source)
        .ok_or_else(|| "ServiceWorker source wrapper is unavailable".to_owned())?;
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ServiceWorker realm wrapper".to_owned());
    }
    super::event_target::attach(scope, object);
    let record = WorkerRecord {
        object: v8::Global::new(scope, object),
        context: v8::Global::new(scope, scope.get_current_context()),
        logical_id: source.logical_id,
        interface_realm_id: crate::webidl::realm_id(scope),
        url: source.url,
        state: source.state,
        container_id: source.container_id,
        worker_realm_id: source.worker_realm_id,
        on_state: None,
        on_error: None,
    };
    scope
        .get_slot_mut::<ServiceWorkerStore>()
        .ok_or_else(|| "ServiceWorker state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<WorkerRecord> {
    scope
        .get_slot::<ServiceWorkerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) enum TextProperty {
    ScriptUrl,
    State,
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
    let text = match property {
        TextProperty::ScriptUrl => record.url,
        TextProperty::State => record.state,
    };
    if let Some(text) = v8::String::new(scope, &text) {
        result.set(text.into());
    }
}

#[derive(Clone, Copy)]
pub(crate) enum HandlerKind {
    StateChange,
    Error,
}

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    kind: HandlerKind,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let handler = match kind {
        HandlerKind::StateChange => record.on_state,
        HandlerKind::Error => record.on_error,
    };
    super::window_event_handler_support::return_handler(scope, handler, result);
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    kind: HandlerKind,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<ServiceWorkerStore>()
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
        HandlerKind::StateChange => record.on_state = handler,
        HandlerKind::Error => record.on_error = handler,
    }
}

pub(crate) fn post_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'postMessage': 1 argument required",
        );
        return;
    }
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(realm_id) = record
        .worker_realm_id
        .filter(|_| record.state == "activated")
    else {
        return;
    };
    let Ok(message) =
        super::worker_structured_clone::serialize(scope, arguments.get(0), arguments.get(1))
    else {
        return;
    };
    super::worker_global_scope::deliver_service_message(
        scope,
        realm_id,
        record.container_id,
        &message,
    );
}

pub(crate) fn activate(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    realm_id: i32,
) {
    let logical_id = record(scope, object).map(|record| record.logical_id);
    if let Some(logical_id) = logical_id
        && let Some(store) = scope.get_slot_mut::<ServiceWorkerStore>()
    {
        for record in store
            .records
            .values_mut()
            .filter(|record| record.logical_id == logical_id)
        {
            record.worker_realm_id = Some(realm_id);
        }
    }
    set_state(scope, object, "installed");
    set_state(scope, object, "activating");
    set_state(scope, object, "activated");
}

pub(crate) fn make_redundant(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    set_state(scope, object, "redundant");
}

pub(crate) fn realm_id(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    record(scope, object)?.worker_realm_id
}

pub(crate) fn logical_id(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    record(scope, object).map(|record| record.logical_id)
}

pub(crate) fn wrapper_for_realm(
    scope: &v8::PinScope<'_, '_>,
    logical_id: i32,
    interface_realm_id: i32,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<ServiceWorkerStore>()?
        .records
        .values()
        .find(|record| {
            record.logical_id == logical_id && record.interface_realm_id == interface_realm_id
        })
        .map(|record| record.object.clone())
}

fn set_state(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, state: &str) {
    let Some(logical_id) = record(scope, object).map(|record| record.logical_id) else {
        return;
    };
    let aliases = {
        let Some(store) = scope.get_slot_mut::<ServiceWorkerStore>() else {
            return;
        };
        store
            .records
            .values_mut()
            .filter(|record| record.logical_id == logical_id)
            .map(|record| {
                record.state = state.to_owned();
                (
                    record.context.clone(),
                    record.object.clone(),
                    record.on_state.clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    for (context, object, handler) in aliases {
        let context = v8::Local::new(scope, &context);
        let realm_scope = &mut v8::ContextScope::new(scope, context);
        let object = v8::Local::new(realm_scope, &object);
        let event = super::event_target::create_event(realm_scope, "statechange");
        super::event_target::dispatch(realm_scope, object, event);
        if let Some(handler) = handler
            && let Ok(handler) =
                v8::Local::<v8::Function>::try_from(v8::Local::new(realm_scope, &handler))
        {
            let _ = handler.call(realm_scope, object.into(), &[event.into()]);
        }
    }
}

pub(crate) fn dispatch_error(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    message: String,
) {
    let handler = record(scope, object).and_then(|record| record.on_error);
    let error = v8::String::new(scope, &message)
        .map(v8::Local::<v8::Value>::from)
        .unwrap_or_else(|| v8::undefined(scope).into());
    let Ok(event) = super::error_event::create(scope, "error", message, error) else {
        return;
    };
    super::event_target::dispatch(scope, object, event);
    if let Some(handler) = handler
        && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = handler.call(scope, object.into(), &[event.into()]);
    }
}
