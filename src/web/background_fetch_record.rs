use std::collections::HashMap;

#[derive(Clone)]
struct BackgroundFetchRecordData {
    request: v8::Global<v8::Object>,
    response_ready: v8::Global<v8::Promise>,
}

#[derive(Default)]
pub(crate) struct BackgroundFetchRecordStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BackgroundFetchRecordData>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BackgroundFetchRecordStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BackgroundFetchRecord", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<BackgroundFetchRecordStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BackgroundFetchRecord",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "request", get_request)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "responseReady", get_response_ready)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BackgroundFetchRecordStore>()
        .ok_or_else(|| "BackgroundFetchRecord state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    request: v8::Local<'_, v8::Object>,
    response: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create BackgroundFetchRecord".to_owned());
    }
    let response_ready = super::writable_stream::resolved_promise(scope, response.into())?;
    let record = BackgroundFetchRecordData {
        request: v8::Global::new(scope, request),
        response_ready: v8::Global::new(scope, response_ready),
    };
    scope
        .get_slot_mut::<BackgroundFetchRecordStore>()
        .ok_or_else(|| "BackgroundFetchRecord state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn request(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<BackgroundFetchRecordStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .map(|record| record.request.clone())
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'BackgroundFetchRecord': Illegal constructor",
    );
}

fn get_request(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(request) = request(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &request).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_response_ready(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let response = scope
        .get_slot::<BackgroundFetchRecordStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.response_ready.clone());
    if let Some(response) = response {
        result.set(v8::Local::new(scope, &response).into());
    } else {
        if let Some(promise) = crate::webidl::rejected_type_error_promise(
            scope,
            "Failed to read the 'responseReady' property from 'BackgroundFetchRecord': Illegal invocation",
        ) {
            result.set(promise.into());
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<BackgroundFetchRecordStore>() {
        store.constructor.remove(realm_id);
    }
}
