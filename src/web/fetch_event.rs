use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct FetchEventRecord {
    pub(crate) request: v8::Global<v8::Object>,
    pub(crate) client_id: String,
    pub(crate) resulting_client_id: String,
    pub(crate) replaces_client_id: String,
    pub(crate) handled: v8::Global<v8::Promise>,
    pub(crate) preload_response: v8::Global<v8::Promise>,
    pub(crate) response: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct FetchEventStore {
    constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, FetchEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FetchEventStore::default());
}

pub(crate) fn install_in_service_worker_realm(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FetchEvent", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FetchEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FetchEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::extendable_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::fetch_event_request_property::define(scope, prototype)?;
    super::fetch_event_preload_response_property::define(scope, prototype)?;
    super::fetch_event_client_id_property::define(scope, prototype)?;
    super::fetch_event_resulting_client_id_property::define(scope, prototype)?;
    super::fetch_event_replaces_client_id_property::define(scope, prototype)?;
    super::fetch_event_handled_property::define(scope, prototype)?;
    super::fetch_event_respond_with::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FetchEventStore>()
        .ok_or_else(|| "FetchEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "FetchEvent requires a type and request");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "FetchEventInit is required");
        return;
    };
    let Some(request_key) = v8::String::new(scope, "request") else {
        return;
    };
    let Some(request_value) = init.get(scope, request_key.into()) else {
        return;
    };
    let request = match super::request::create_from_input(scope, request_value) {
        Ok(request) => request,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if attach_record(scope, arguments.this(), &event_type, request, "", "", "").is_err() {
        crate::webidl::throw_type_error(scope, "cannot construct FetchEvent");
        return;
    }
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    request: v8::Local<'s, v8::Object>,
    client_id: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create FetchEvent".to_owned());
    }
    attach_record(scope, event, "fetch", request, client_id, "", "")?;
    Ok(event)
}

fn attach_record(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
    request: v8::Local<'_, v8::Object>,
    client_id: &str,
    resulting_client_id: &str,
    replaces_client_id: &str,
) -> Result<(), String> {
    super::event::attach(scope, event, event_type.to_owned(), false, true, false);
    scope
        .get_slot_mut::<super::extendable_event::ExtendableEventStore>()
        .ok_or_else(|| "ExtendableEvent state was not prepared".to_owned())?
        .records
        .insert(event.get_identity_hash().get(), Vec::new());
    let undefined = v8::undefined(scope);
    let handled = super::writable_stream::resolved_promise(scope, undefined.into())?;
    let preload_response = super::writable_stream::resolved_promise(scope, undefined.into())?;
    let record = FetchEventRecord {
        request: v8::Global::new(scope, request),
        client_id: client_id.to_owned(),
        resulting_client_id: resulting_client_id.to_owned(),
        replaces_client_id: replaces_client_id.to_owned(),
        handled: v8::Global::new(scope, handled),
        preload_response: v8::Global::new(scope, preload_response),
        response: None,
    };
    scope
        .get_slot_mut::<FetchEventStore>()
        .ok_or_else(|| "FetchEvent state was not prepared".to_owned())?
        .records
        .insert(event.get_identity_hash().get(), record);
    Ok(())
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> Option<FetchEventRecord> {
    scope
        .get_slot::<FetchEventStore>()?
        .records
        .get(&event.get_identity_hash().get())
        .cloned()
}

pub(crate) fn take_response(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Value>> {
    scope
        .get_slot_mut::<FetchEventStore>()?
        .records
        .get_mut(&event.get_identity_hash().get())?
        .response
        .take()
}
