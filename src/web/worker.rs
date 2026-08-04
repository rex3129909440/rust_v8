use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WorkerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WorkerRecord>,
}

#[derive(Clone)]
struct WorkerRecord {
    object: v8::Global<v8::Object>,
    parent_context: v8::Global<v8::Context>,
    realm_id: i32,
    terminated: bool,
    on_message: Option<v8::Global<v8::Value>>,
    on_message_error: Option<v8::Global<v8::Value>>,
    on_error: Option<v8::Global<v8::Value>>,
    incoming: Vec<super::worker_structured_clone::SerializedMessage>,
    pending_errors: Vec<super::worker_global_scope::WorkerScriptError>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WorkerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Worker", constructor.into())
}

pub(crate) fn install_in_worker_realm(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = build_constructor(scope)?;
    crate::webidl::define_global(scope, "Worker", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(existing) = scope
        .get_slot::<WorkerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = build_constructor(scope)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WorkerStore>()
        .ok_or_else(|| "Worker state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn build_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let event_target = global_function(scope, "EventTarget")
        .or_else(|_| super::event_target::ensure_constructor(scope))?;
    let constructor = crate::webidl::create_function(
        scope,
        "Worker",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::worker_onmessage_property::define(scope, prototype)?;
    super::worker_post_message::define(scope, prototype)?;
    super::worker_terminate::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::worker_onerror_property::define(scope, prototype)?;
    Ok(constructor)
}

fn global_function<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, name)?;
    let value = global
        .get(scope, key.into())
        .ok_or_else(|| format!("{name} is unavailable"))?;
    v8::Local::<v8::Function>::try_from(value).map_err(|_| format!("{name} is not a function"))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'Worker': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Worker': 1 argument required");
        return;
    }
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = match read_options(scope, arguments.get(1)) {
        Ok(options) => options,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let script = match super::worker_script_source::load(scope, &input, None) {
        Ok(script) => script,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let object = arguments.this();
    let object_id = object.get_identity_hash().get();
    super::event_target::attach(scope, object);
    let realm_id = match super::worker_global_scope::create(
        scope,
        super::worker_global_scope::RealmOwner::Dedicated(object_id),
        super::worker_global_scope::RealmKind::Dedicated,
        script.url,
        options.name,
        options.module,
    ) {
        Ok(realm_id) => realm_id,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let record = WorkerRecord {
        object: v8::Global::new(scope, object),
        parent_context: v8::Global::new(scope, scope.get_entered_or_microtask_context()),
        realm_id,
        terminated: false,
        on_message: None,
        on_message_error: None,
        on_error: None,
        incoming: Vec::new(),
        pending_errors: Vec::new(),
    };
    scope
        .get_slot_mut::<WorkerStore>()
        .expect("Worker state")
        .records
        .insert(object_id, record);
    if let Err(error) = super::worker_global_scope::evaluate(scope, realm_id, &script.source) {
        let canceled = super::worker_global_scope::dispatch_script_error(scope, realm_id, &error);
        if !canceled
            && let Some(record) = scope
                .get_slot_mut::<WorkerStore>()
                .and_then(|store| store.records.get_mut(&object_id))
        {
            record.pending_errors.push(error);
        }
    }
    result.set(object.into());
}

struct WorkerOptions {
    name: String,
    module: bool,
}

fn read_options(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<WorkerOptions, String> {
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        return Ok(WorkerOptions {
            name: String::new(),
            module: false,
        });
    };
    let name = string_property(scope, options, "name").unwrap_or_default();
    let worker_type =
        string_property(scope, options, "type").unwrap_or_else(|| "classic".to_owned());
    if worker_type != "classic" && worker_type != "module" {
        return Err(format!("'{worker_type}' is not a valid Worker type"));
    }
    if let Some(credentials) = string_property(scope, options, "credentials")
        && credentials != "omit"
        && credentials != "same-origin"
        && credentials != "include"
    {
        return Err(format!("'{credentials}' is not a valid credentials mode"));
    }
    Ok(WorkerOptions {
        name,
        module: worker_type == "module",
    })
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    object
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<WorkerRecord> {
    scope
        .get_slot::<WorkerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn realm_id_for(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    record(scope, object)
        .filter(|record| !record.terminated)
        .map(|record| record.realm_id)
}

#[derive(Clone, Copy)]
pub(crate) enum HandlerKind {
    Message,
    MessageError,
    Error,
}

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    kind: HandlerKind,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let handler = match kind {
        HandlerKind::Message => record.on_message,
        HandlerKind::MessageError => record.on_message_error,
        HandlerKind::Error => record.on_error,
    };
    if let Some(handler) = handler {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    kind: HandlerKind,
) {
    let handler = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|function| v8::Global::new(scope, v8::Local::<v8::Value>::from(function)));
    let id = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<WorkerStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match kind {
        HandlerKind::Message => record.on_message = handler,
        HandlerKind::MessageError => record.on_message_error = handler,
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
            "Failed to execute 'postMessage' on 'Worker': 1 argument required",
        );
        return;
    }
    let id = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot::<WorkerStore>()
        .and_then(|store| store.records.get(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.terminated {
        return;
    }
    let Ok(message) =
        super::worker_structured_clone::serialize(scope, arguments.get(0), arguments.get(1))
    else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<WorkerStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.incoming.push(message);
    }
}

pub(crate) fn terminate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<WorkerStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.terminated = true;
    record.incoming.clear();
    record.pending_errors.clear();
    let realm_id = record.realm_id;
    super::worker_global_scope::terminate_realm(scope, realm_id);
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let incoming = take_incoming(scope);
    let errors = take_errors(scope);
    let mut had_work = !incoming.is_empty() || !errors.is_empty();
    for (realm_id, message) in incoming {
        let _ = super::worker_global_scope::deliver_message(scope, realm_id, &message);
    }
    had_work |= super::worker_global_scope::run_timers(scope);
    let outgoing = super::worker_global_scope::take_outgoing(scope);
    had_work |= !outgoing.is_empty();
    for (owner, message) in outgoing {
        match owner {
            super::worker_global_scope::RealmOwner::Dedicated(worker_id) => {
                deliver_to_parent(scope, worker_id, message);
            }
            super::worker_global_scope::RealmOwner::Shared(runtime_id) => {
                super::shared_worker::discard_unaddressed_outgoing(scope, runtime_id, message);
            }
            super::worker_global_scope::RealmOwner::Service(_) => {}
        }
    }
    for (worker_id, message) in errors {
        deliver_error_to_parent(scope, worker_id, &message);
    }
    super::shared_worker::run_pending_tasks(scope);
    super::worker_global_scope::reclaim_closed_realms(scope);
    had_work
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    super::worker_global_scope::next_due(scope)
}

pub(crate) fn terminate_children_for_parent_context(
    scope: &mut v8::PinScope<'_, '_>,
    parent_context: &v8::Global<v8::Context>,
) {
    let parent_context = v8::Local::new(scope, parent_context);
    let children = scope
        .get_slot::<WorkerStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter_map(|(worker_id, record)| {
                    let record_context = v8::Local::new(scope, &record.parent_context);
                    (!record.terminated && record_context == parent_context).then_some(*worker_id)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for worker_id in children {
        let realm_id = scope
            .get_slot_mut::<WorkerStore>()
            .and_then(|store| store.records.get_mut(&worker_id))
            .map(|record| {
                record.terminated = true;
                record.incoming.clear();
                record.pending_errors.clear();
                record.realm_id
            });
        if let Some(realm_id) = realm_id {
            super::worker_global_scope::terminate_realm(scope, realm_id);
        }
    }
}

fn take_incoming(
    scope: &mut v8::PinScope<'_, '_>,
) -> Vec<(i32, super::worker_structured_clone::SerializedMessage)> {
    let Some(store) = scope.get_slot_mut::<WorkerStore>() else {
        return Vec::new();
    };
    let mut messages = Vec::new();
    for record in store
        .records
        .values_mut()
        .filter(|record| !record.terminated)
    {
        let realm_id = record.realm_id;
        messages.extend(
            std::mem::take(&mut record.incoming)
                .into_iter()
                .map(|message| (realm_id, message)),
        );
    }
    messages.sort_by_key(|(realm_id, _)| *realm_id);
    messages
}

fn take_errors(
    scope: &mut v8::PinScope<'_, '_>,
) -> Vec<(i32, super::worker_global_scope::WorkerScriptError)> {
    let Some(store) = scope.get_slot_mut::<WorkerStore>() else {
        return Vec::new();
    };
    let mut errors = Vec::new();
    for (worker_id, record) in store
        .records
        .iter_mut()
        .filter(|(_, record)| !record.terminated)
    {
        errors.extend(
            std::mem::take(&mut record.pending_errors)
                .into_iter()
                .map(|error| (*worker_id, error)),
        );
    }
    errors.sort_by_key(|(worker_id, _)| *worker_id);
    errors
}

fn deliver_to_parent(
    scope: &mut v8::PinScope<'_, '_>,
    worker_id: i32,
    message: super::worker_structured_clone::SerializedMessage,
) {
    let Some(record) = scope
        .get_slot::<WorkerStore>()
        .and_then(|store| store.records.get(&worker_id))
        .cloned()
        .filter(|record| !record.terminated)
    else {
        return;
    };
    let context = v8::Local::new(scope, &record.parent_context);
    let parent_scope = &mut v8::ContextScope::new(scope, context);
    let Some(data) = super::worker_structured_clone::deserialize(parent_scope, &message) else {
        let event = super::event_target::create_event(parent_scope, "messageerror");
        let handler = record.on_message_error.clone();
        dispatch_parent_event(parent_scope, &record, event, handler);
        return;
    };
    let ports = message
        .ports
        .iter()
        .map(|port| v8::Local::new(parent_scope, port))
        .collect();
    let Ok(event) = super::message_event::create(parent_scope, "message", data, "", None, ports)
    else {
        return;
    };
    let handler = record.on_message.clone();
    dispatch_parent_event(parent_scope, &record, event, handler);
}

fn deliver_error_to_parent(
    scope: &mut v8::PinScope<'_, '_>,
    worker_id: i32,
    detail: &super::worker_global_scope::WorkerScriptError,
) {
    let Some(record) = scope
        .get_slot::<WorkerStore>()
        .and_then(|store| store.records.get(&worker_id))
        .cloned()
        .filter(|record| !record.terminated)
    else {
        return;
    };
    let context = v8::Local::new(scope, &record.parent_context);
    let parent_scope = &mut v8::ContextScope::new(scope, context);
    let error = v8::Exception::error(
        parent_scope,
        v8::String::new(parent_scope, &detail.message).expect("Worker error"),
    );
    let Ok(event) = super::error_event::create_detailed(
        parent_scope,
        "error",
        detail.message.clone(),
        detail.filename.clone(),
        detail.lineno,
        detail.colno,
        error,
    ) else {
        return;
    };
    let handler = record.on_error.clone();
    dispatch_parent_event(parent_scope, &record, event, handler);
}

fn dispatch_parent_event(
    scope: &mut v8::PinScope<'_, '_>,
    record: &WorkerRecord,
    event: v8::Local<'_, v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
) {
    let target = v8::Local::new(scope, &record.object);
    super::event_target::dispatch(scope, target, event);
    if let Some(handler) = handler
        && let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = function.call(scope, target.into(), &[event.into()]);
    }
}
