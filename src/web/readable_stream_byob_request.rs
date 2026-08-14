use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ReadableStreamByobRequestStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RequestRecord>,
    active_by_stream: HashMap<i32, v8::Global<v8::Object>>,
}

struct RequestRecord {
    stream_identity: i32,
    view: Option<v8::Global<v8::Object>>,
    resolver: Option<v8::Global<v8::PromiseResolver>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ReadableStreamByobRequestStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ReadableStreamBYOBRequest", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ReadableStreamByobRequestStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ReadableStreamBYOBRequest",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "view", get_view)?;
    crate::webidl::define_method(scope, prototype, "respond", 1, respond)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "respondWithNewView",
        1,
        respond_with_new_view,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ReadableStreamByobRequestStore>()
        .ok_or_else(|| "ReadableStreamBYOBRequest state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    stream: v8::Local<'_, v8::Object>,
    view: v8::Local<'_, v8::Object>,
    resolver: v8::Local<'_, v8::PromiseResolver>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let request = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, request, prototype.into()) != Some(true) {
        return Err("cannot create ReadableStreamBYOBRequest".to_owned());
    }
    let stream_identity = stream.get_identity_hash().get();
    let request_identity = request.get_identity_hash().get();
    let request_global = v8::Global::new(scope, request);
    let record = RequestRecord {
        stream_identity,
        view: Some(v8::Global::new(scope, view)),
        resolver: Some(v8::Global::new(scope, resolver)),
    };
    let store = scope
        .get_slot_mut::<ReadableStreamByobRequestStore>()
        .ok_or_else(|| "ReadableStreamBYOBRequest state was not prepared".to_owned())?;
    store.records.insert(request_identity, record);
    store
        .active_by_stream
        .insert(stream_identity, request_global);
    Ok(request)
}

pub(crate) fn current_for_stream<'s>(
    scope: &v8::PinScope<'s, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    scope
        .get_slot::<ReadableStreamByobRequestStore>()?
        .active_by_stream
        .get(&stream.get_identity_hash().get())
        .map(|request| v8::Local::new(scope, request))
}

pub(crate) fn fulfill_stream(
    scope: &mut v8::PinScope<'_, '_>,
    stream: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
) -> bool {
    let Some(request) = current_for_stream(scope, stream) else {
        return false;
    };
    resolve_request(scope, request, value, false)
}

pub(crate) fn close_stream(
    scope: &mut v8::PinScope<'_, '_>,
    stream: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(request) = current_for_stream(scope, stream) else {
        return false;
    };
    let value = v8::undefined(scope);
    resolve_request(scope, request, value.into(), true)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ReadableStreamBYOBRequest': Illegal constructor",
    );
}

fn get_view(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot::<ReadableStreamByobRequestStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.view.as_ref() {
        Some(view) => result.set(v8::Local::new(scope, view).into()),
        None => result.set(v8::null(scope).into()),
    }
}

fn respond(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if scope
        .get_slot::<ReadableStreamByobRequestStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let written = arguments.get(0).uint32_value(scope).unwrap_or(0);
    if written == 0 {
        crate::webidl::throw_type_error(scope, "bytes written is 0");
        return;
    }
    let view = scope
        .get_slot::<ReadableStreamByobRequestStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .and_then(|record| record.view.as_ref())
        .cloned();
    let Some(view) = view else {
        crate::webidl::throw_type_error(scope, "This BYOB request has been invalidated");
        return;
    };
    let view = v8::Local::new(scope, &view);
    let _ = resolve_request(scope, arguments.this(), view.into(), false);
}

fn respond_with_new_view(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if scope
        .get_slot::<ReadableStreamByobRequestStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(view) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "respondWithNewView requires an ArrayBufferView");
        return;
    };
    if !resolve_request(scope, arguments.this(), view.into(), false) {
        crate::webidl::throw_type_error(scope, "This BYOB request has been invalidated");
    }
}

fn resolve_request(
    scope: &mut v8::PinScope<'_, '_>,
    request: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    done: bool,
) -> bool {
    let identity = request.get_identity_hash().get();
    let Some((stream_identity, resolver)) = scope
        .get_slot_mut::<ReadableStreamByobRequestStore>()
        .and_then(|store| {
            let record = store.records.get_mut(&identity)?;
            let resolver = record.resolver.take()?;
            record.view = None;
            Some((record.stream_identity, resolver))
        })
    else {
        return false;
    };
    if let Some(store) = scope.get_slot_mut::<ReadableStreamByobRequestStore>() {
        store.active_by_stream.remove(&stream_identity);
    }
    let output = v8::Object::new(scope);
    define_data(scope, output, "value", value);
    define_data(scope, output, "done", v8::Boolean::new(scope, done).into());
    let resolver = v8::Local::new(scope, &resolver);
    let _ = resolver.resolve(scope, output.into());
    true
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ReadableStreamByobRequestStore>() {
        store.constructor.remove(realm_id);
    }
}
