use std::collections::HashMap;

#[derive(Clone)]
struct ClientRecord {
    container_id: i32,
    id: String,
    url: String,
    focused: bool,
    object: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct ServiceWorkerClientsStore {
    clients_objects: HashMap<i32, i32>,
    client_records: HashMap<i32, ClientRecord>,
    clients_by_container: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ServiceWorkerClientsStore::default());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    container_id: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let event_target = global_function(scope, "EventTarget")?;
    let client = crate::webidl::create_function(
        scope,
        "Client",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_client,
    )?;
    crate::webidl::inherit(scope, client, event_target)?;
    let client_prototype = crate::webidl::prototype(scope, client)?;
    crate::webidl::reset_constructor_order(scope, client_prototype)?;
    super::service_worker_client_id_property::define(scope, client_prototype)?;
    super::service_worker_client_type_property::define(scope, client_prototype)?;
    super::service_worker_client_url_property::define(scope, client_prototype)?;
    super::service_worker_client_post_message::define(scope, client_prototype)?;
    crate::webidl::finish_constructor(scope, client_prototype, client)?;
    crate::webidl::define_global(scope, "Client", client.into())?;

    let window_client = crate::webidl::create_function(
        scope,
        "WindowClient",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_window_client,
    )?;
    crate::webidl::inherit(scope, window_client, client)?;
    let window_client_prototype = crate::webidl::prototype(scope, window_client)?;
    crate::webidl::reset_constructor_order(scope, window_client_prototype)?;
    super::service_worker_window_client_ancestor_origins_property::define(
        scope,
        window_client_prototype,
    )?;
    super::service_worker_window_client_focused_property::define(scope, window_client_prototype)?;
    super::service_worker_window_client_visibility_state_property::define(
        scope,
        window_client_prototype,
    )?;
    super::service_worker_window_client_focus::define(scope, window_client_prototype)?;
    super::service_worker_window_client_navigate::define(scope, window_client_prototype)?;
    crate::webidl::finish_constructor(scope, window_client_prototype, window_client)?;
    crate::webidl::define_global(scope, "WindowClient", window_client.into())?;

    let clients = crate::webidl::create_function(
        scope,
        "Clients",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_clients,
    )?;
    let clients_prototype = crate::webidl::prototype(scope, clients)?;
    crate::webidl::reset_constructor_order(scope, clients_prototype)?;
    super::service_worker_clients_get::define(scope, clients_prototype)?;
    super::service_worker_clients_match_all::define(scope, clients_prototype)?;
    super::service_worker_clients_open_window::define(scope, clients_prototype)?;
    super::service_worker_clients_claim::define(scope, clients_prototype)?;
    crate::webidl::finish_constructor(scope, clients_prototype, clients)?;
    crate::webidl::define_global(scope, "Clients", clients.into())?;

    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, clients_prototype.into()) != Some(true)
    {
        return Err("cannot create Clients".to_owned());
    }
    let window = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, window, window_client_prototype.into())
        != Some(true)
    {
        return Err("cannot create WindowClient".to_owned());
    }
    let window_global = v8::Global::new(scope, window);
    let record = ClientRecord {
        container_id,
        id: format!("edge-window-client-{container_id}"),
        url: "https://sandbox.test/".to_owned(),
        focused: true,
        object: window_global.clone(),
    };
    let store = scope
        .get_slot_mut::<ServiceWorkerClientsStore>()
        .ok_or_else(|| "Service worker Clients state was not prepared".to_owned())?;
    store
        .clients_objects
        .insert(object.get_identity_hash().get(), container_id);
    store
        .client_records
        .insert(window.get_identity_hash().get(), record);
    store
        .clients_by_container
        .insert(container_id, window_global);
    Ok(object)
}

fn global_function<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let key = crate::webidl::string(scope, name)?;
    let value = scope
        .get_current_context()
        .global(scope)
        .get(scope, key.into())
        .ok_or_else(|| format!("{name} is unavailable"))?;
    v8::Local::<v8::Function>::try_from(value).map_err(|_| format!("{name} is not a function"))
}

fn illegal_client(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

fn illegal_window_client(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

fn illegal_clients(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn client_for_container<'s>(
    scope: &v8::PinScope<'s, '_>,
    container_id: i32,
) -> Option<v8::Local<'s, v8::Object>> {
    let client = scope
        .get_slot::<ServiceWorkerClientsStore>()?
        .clients_by_container
        .get(&container_id)?
        .clone();
    Some(v8::Local::new(scope, &client))
}

fn client_record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ClientRecord> {
    scope
        .get_slot::<ServiceWorkerClientsStore>()?
        .client_records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn clients_container(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    scope
        .get_slot::<ServiceWorkerClientsStore>()?
        .clients_objects
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn get_client_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    property: ClientTextProperty,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = client_record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = match property {
        ClientTextProperty::Id => record.id,
        ClientTextProperty::Type => "window".to_owned(),
        ClientTextProperty::Url => record.url,
        ClientTextProperty::VisibilityState => "visible".to_owned(),
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) enum ClientTextProperty {
    Id,
    Type,
    Url,
    VisibilityState,
}

pub(crate) fn get_client_focused(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = client_record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.focused).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_ancestor_origins(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if client_record(scope, arguments.this()).is_some() {
        result.set(v8::Array::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn post_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let Some(record) = client_record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(message) =
        super::worker_structured_clone::serialize(scope, arguments.get(0), arguments.get(1))
    else {
        return;
    };
    super::service_worker_container::queue_from_service(scope, record.container_id, message);
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

pub(crate) fn focus(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let object = {
        let Some(record) = scope
            .get_slot_mut::<ServiceWorkerClientsStore>()
            .and_then(|store| store.client_records.get_mut(&id))
        else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        };
        record.focused = true;
        record.object.clone()
    };
    let object = v8::Local::new(scope, &object);
    resolved(scope, object.into(), result);
}

pub(crate) fn navigate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let url = crate::webidl::value_to_string(scope, arguments.get(0));
    let id = arguments.this().get_identity_hash().get();
    let object = {
        let Some(record) = scope
            .get_slot_mut::<ServiceWorkerClientsStore>()
            .and_then(|store| store.client_records.get_mut(&id))
        else {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
            return;
        };
        record.url = url;
        record.object.clone()
    };
    let object = v8::Local::new(scope, &object);
    resolved(scope, object.into(), result);
}

pub(crate) fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(container_id) = clients_container(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let requested = crate::webidl::value_to_string(scope, arguments.get(0));
    let client = client_for_container(scope, container_id);
    let value = client
        .filter(|client| client_record(scope, *client).is_some_and(|record| record.id == requested))
        .map(v8::Local::<v8::Value>::from)
        .unwrap_or_else(|| v8::undefined(scope).into());
    resolved(scope, value, result);
}

pub(crate) fn match_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(container_id) = clients_container(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, 1);
    if let Some(client) = client_for_container(scope, container_id) {
        let _ = array.set_index(scope, 0, client.into());
    }
    resolved(scope, array.into(), result);
}

pub(crate) fn open_window(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(container_id) = clients_container(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let url = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(client) = client_for_container(scope, container_id) else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<ServiceWorkerClientsStore>()
        .and_then(|store| {
            store
                .client_records
                .get_mut(&client.get_identity_hash().get())
        })
    {
        record.url = url;
        record.focused = true;
    }
    resolved(scope, client.into(), result);
}

pub(crate) fn claim(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if clients_container(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    resolved(scope, v8::undefined(scope).into(), result);
}
