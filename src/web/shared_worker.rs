use std::collections::HashMap;

#[derive(Clone)]
struct SharedWorkerRecord {
    port: v8::Global<v8::Object>,
    runtime_id: u64,
    parent_context: v8::Global<v8::Context>,
    object: v8::Global<v8::Object>,
    onerror: Option<v8::Global<v8::Value>>,
    pending_errors: Vec<super::worker_global_scope::WorkerScriptError>,
}

#[derive(Clone)]
struct SharedRuntime {
    key: String,
    realm_id: i32,
    ports: Vec<v8::Global<v8::Object>>,
}

pub(crate) struct SharedWorkerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SharedWorkerRecord>,
    runtimes: HashMap<u64, SharedRuntime>,
    runtime_by_key: HashMap<String, u64>,
    next_runtime_id: u64,
}

impl Default for SharedWorkerStore {
    fn default() -> Self {
        Self {
            constructor: crate::webidl::RealmConstructor::default(),
            records: HashMap::new(),
            runtimes: HashMap::new(),
            runtime_by_key: HashMap::new(),
            next_runtime_id: 1,
        }
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SharedWorkerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SharedWorker", constructor.into())
}

pub(crate) fn install_in_worker_realm(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = build_constructor(scope)?;
    crate::webidl::define_global(scope, "SharedWorker", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<SharedWorkerStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = build_constructor(scope)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SharedWorkerStore>()
        .ok_or_else(|| "SharedWorker state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

fn build_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let constructor = crate::webidl::create_function(
        scope,
        "SharedWorker",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::shared_worker_port_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::shared_worker_onerror_property::define(scope, prototype)?;
    let event_target = global_function(scope, "EventTarget")
        .or_else(|_| super::event_target::ensure_constructor(scope))?;
    crate::webidl::inherit(scope, constructor, event_target)?;
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
            "Failed to construct 'SharedWorker': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SharedWorker': 1 argument required, but only 0 present.",
        );
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
    let creator_context = scope.get_entered_or_microtask_context();
    let creator_window = creator_context.global(scope);
    let creator_origin = super::html_i_frame_element::origin_for_window(scope, creator_window);
    let key = format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}",
        creator_origin, script.url, options.name, options.worker_type
    );
    let runtime = existing_runtime(scope, &key);
    let (runtime_id, realm_id, newly_created) = if let Some(runtime) = runtime {
        (runtime.0, runtime.1, false)
    } else {
        let runtime_id = reserve_runtime_id(scope);
        let realm_id = match super::worker_global_scope::create(
            scope,
            super::worker_global_scope::RealmOwner::Shared(runtime_id),
            super::worker_global_scope::RealmKind::Shared,
            script.url.clone(),
            options.name.clone(),
            options.worker_type == "module",
        ) {
            Ok(realm_id) => realm_id,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        let runtime = SharedRuntime {
            key: key.clone(),
            realm_id,
            ports: Vec::new(),
        };
        let store = scope
            .get_slot_mut::<SharedWorkerStore>()
            .expect("SharedWorker state");
        store.runtimes.insert(runtime_id, runtime);
        store.runtime_by_key.insert(key, runtime_id);
        (runtime_id, realm_id, true)
    };
    let (port, worker_port) = match super::message_port::create_pair(scope) {
        Ok(pair) => pair,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let object = arguments.this();
    super::event_target::attach(scope, object);
    let object_id = object.get_identity_hash().get();
    let record = SharedWorkerRecord {
        port: v8::Global::new(scope, port),
        runtime_id,
        parent_context: v8::Global::new(scope, scope.get_current_context()),
        object: v8::Global::new(scope, object),
        onerror: None,
        pending_errors: Vec::new(),
    };
    scope
        .get_slot_mut::<SharedWorkerStore>()
        .expect("SharedWorker state")
        .records
        .insert(object_id, record);
    let parent_port = v8::Global::new(scope, port);
    let worker_port_global = v8::Global::new(scope, worker_port);
    if let Some(runtime) = scope
        .get_slot_mut::<SharedWorkerStore>()
        .and_then(|store| store.runtimes.get_mut(&runtime_id))
    {
        runtime.ports.push(parent_port);
        runtime.ports.push(worker_port_global.clone());
    }
    if newly_created
        && let Err(error) = super::worker_global_scope::evaluate(scope, realm_id, &script.source)
    {
        let canceled = super::worker_global_scope::dispatch_script_error(scope, realm_id, &error);
        if !canceled
            && let Some(record) = scope
                .get_slot_mut::<SharedWorkerStore>()
                .and_then(|store| store.records.get_mut(&object_id))
        {
            record.pending_errors.push(error);
        }
    }
    let _ = super::worker_global_scope::dispatch_connect(scope, realm_id, worker_port_global);
    result.set(object.into());
}

struct SharedOptions {
    name: String,
    worker_type: String,
}

fn read_options(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<SharedOptions, String> {
    if value.is_string() {
        return Ok(SharedOptions {
            name: crate::webidl::value_to_string(scope, value),
            worker_type: "classic".to_owned(),
        });
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        return Ok(SharedOptions {
            name: String::new(),
            worker_type: "classic".to_owned(),
        });
    };
    let name = string_property(scope, options, "name").unwrap_or_default();
    let worker_type =
        string_property(scope, options, "type").unwrap_or_else(|| "classic".to_owned());
    if worker_type != "classic" && worker_type != "module" {
        return Err(format!("'{worker_type}' is not a valid SharedWorker type"));
    }
    if let Some(credentials) = string_property(scope, options, "credentials")
        && credentials != "omit"
        && credentials != "same-origin"
        && credentials != "include"
    {
        return Err(format!("'{credentials}' is not a valid credentials mode"));
    }
    Ok(SharedOptions { name, worker_type })
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

fn reserve_runtime_id(scope: &mut v8::PinScope<'_, '_>) -> u64 {
    let store = scope
        .get_slot_mut::<SharedWorkerStore>()
        .expect("SharedWorker state");
    let id = store.next_runtime_id;
    store.next_runtime_id = store.next_runtime_id.saturating_add(1).max(1);
    id
}

fn existing_runtime(scope: &mut v8::PinScope<'_, '_>, key: &str) -> Option<(u64, i32)> {
    let runtime_id = scope
        .get_slot::<SharedWorkerStore>()?
        .runtime_by_key
        .get(key)
        .copied()?;
    let runtime = scope
        .get_slot::<SharedWorkerStore>()?
        .runtimes
        .get(&runtime_id)
        .cloned()?;
    if super::worker_global_scope::is_closed(scope, runtime.realm_id) {
        close_runtime_ports(scope, &runtime);
        if let Some(store) = scope.get_slot_mut::<SharedWorkerStore>() {
            store.runtime_by_key.remove(&runtime.key);
            store.runtimes.remove(&runtime_id);
        }
        None
    } else {
        Some((runtime_id, runtime.realm_id))
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SharedWorkerRecord> {
    scope
        .get_slot::<SharedWorkerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, &record.port).into());
}

pub(crate) fn get_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.onerror {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
) {
    let value = v8::Local::<v8::Function>::try_from(arguments.get(0))
        .ok()
        .map(|function| v8::Global::new(scope, v8::Local::<v8::Value>::from(function)));
    let Some(record) = scope.get_slot_mut::<SharedWorkerStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.onerror = value;
}

pub(crate) fn discard_unaddressed_outgoing(
    scope: &mut v8::PinScope<'_, '_>,
    runtime_id: u64,
    _message: super::worker_structured_clone::SerializedMessage,
) {
    let realm_id = scope
        .get_slot::<SharedWorkerStore>()
        .and_then(|store| store.runtimes.get(&runtime_id))
        .map(|runtime| runtime.realm_id);
    if let Some(realm_id) = realm_id {
        super::worker_global_scope::dispatch_error(
            scope,
            realm_id,
            "SharedWorkerGlobalScope does not provide postMessage; use a connected MessagePort",
        );
    }
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) {
    let closed = scope
        .get_slot::<SharedWorkerStore>()
        .map(|store| {
            store
                .runtimes
                .iter()
                .filter(|(_, runtime)| {
                    super::worker_global_scope::is_closed(scope, runtime.realm_id)
                })
                .map(|(id, runtime)| (*id, runtime.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for (runtime_id, runtime) in closed {
        close_runtime_ports(scope, &runtime);
        if let Some(store) = scope.get_slot_mut::<SharedWorkerStore>() {
            store.runtime_by_key.remove(&runtime.key);
            store.runtimes.remove(&runtime_id);
        }
    }
    let pending = {
        let Some(store) = scope.get_slot_mut::<SharedWorkerStore>() else {
            return;
        };
        let mut pending = Vec::new();
        for record in store.records.values_mut() {
            pending.extend(
                std::mem::take(&mut record.pending_errors)
                    .into_iter()
                    .map(|error| (record.clone(), error)),
            );
        }
        pending
    };
    for (record, detail) in pending {
        let context = v8::Local::new(scope, &record.parent_context);
        let parent_scope = &mut v8::ContextScope::new(scope, context);
        let error = v8::Exception::error(
            parent_scope,
            v8::String::new(parent_scope, &detail.message).expect("SharedWorker error"),
        );
        let Ok(event) = super::error_event::create_detailed(
            parent_scope,
            "error",
            detail.message,
            detail.filename,
            detail.lineno,
            detail.colno,
            error,
        ) else {
            continue;
        };
        let target = v8::Local::new(parent_scope, &record.object);
        super::event_target::dispatch(parent_scope, target, event);
        if let Some(handler) = record.onerror
            && let Ok(function) =
                v8::Local::<v8::Function>::try_from(v8::Local::new(parent_scope, &handler))
        {
            let _ = function.call(parent_scope, target.into(), &[event.into()]);
        }
        let _ = record.runtime_id;
    }
}

fn close_runtime_ports(scope: &mut v8::PinScope<'_, '_>, runtime: &SharedRuntime) {
    for port in &runtime.ports {
        let port = v8::Local::new(scope, port);
        super::message_port::close_object(scope, port);
    }
}
