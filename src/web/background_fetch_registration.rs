use std::collections::HashMap;

#[derive(Clone)]
struct BackgroundFetchRegistrationData {
    id: String,
    upload_total: f64,
    uploaded: f64,
    download_total: f64,
    downloaded: f64,
    result: String,
    failure_reason: String,
    records: Vec<v8::Global<v8::Object>>,
    on_progress: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct BackgroundFetchRegistrationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BackgroundFetchRegistrationData>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BackgroundFetchRegistrationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BackgroundFetchRegistration", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<BackgroundFetchRegistrationStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BackgroundFetchRegistration",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "uploadTotal", get_upload_total)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "uploaded", get_uploaded)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "downloadTotal", get_download_total)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "downloaded", get_downloaded)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "result", get_result)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "failureReason", get_failure_reason)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "recordsAvailable",
        get_records_available,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onprogress",
        get_on_progress,
        set_on_progress,
    )?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::define_method(scope, prototype, "match", 1, match_record)?;
    crate::webidl::define_method(scope, prototype, "matchAll", 0, match_all)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BackgroundFetchRegistrationStore>()
        .ok_or_else(|| "BackgroundFetchRegistration state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    id: String,
    records: Vec<v8::Local<'_, v8::Object>>,
    download_total: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let registration = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, registration, prototype.into()) != Some(true) {
        return Err("cannot create BackgroundFetchRegistration".to_owned());
    }
    super::event_target::attach(scope, registration);
    let data = BackgroundFetchRegistrationData {
        id,
        upload_total: 0.0,
        uploaded: 0.0,
        download_total,
        downloaded: download_total,
        result: "success".to_owned(),
        failure_reason: String::new(),
        records: records
            .into_iter()
            .map(|record| v8::Global::new(scope, record))
            .collect(),
        on_progress: None,
    };
    scope
        .get_slot_mut::<BackgroundFetchRegistrationStore>()
        .ok_or_else(|| "BackgroundFetchRegistration state was not prepared".to_owned())?
        .records
        .insert(registration.get_identity_hash().get(), data);
    Ok(registration)
}

fn data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BackgroundFetchRegistrationData> {
    scope
        .get_slot::<BackgroundFetchRegistrationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut BackgroundFetchRegistrationData),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<BackgroundFetchRegistrationStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    change(record);
    true
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'BackgroundFetchRegistration': Illegal constructor",
    );
}

fn text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&BackgroundFetchRegistrationData) -> &str,
) {
    let Some(record) = data(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

fn number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&BackgroundFetchRegistrationData) -> f64,
) {
    let Some(record) = data(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Number::new(scope, select(&record)).into());
}

fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |x| &x.id)
}
fn get_upload_total(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.upload_total)
}
fn get_uploaded(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.uploaded)
}
fn get_download_total(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.download_total)
}
fn get_downloaded(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number(s, a, r, |x| x.downloaded)
}
fn get_result(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |x| &x.result)
}
fn get_failure_reason(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |x| &x.failure_reason)
}

fn get_records_available(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = data(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, !record.records.is_empty()).into());
}

fn get_on_progress(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = data(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, record.on_progress, result);
}

fn set_on_progress(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if !update(scope, arguments.this(), |record| {
        record.on_progress = handler
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !update(scope, arguments.this(), |record| {
        record.result = "failure".to_owned();
        record.failure_reason = "aborted".to_owned();
    }) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "BackgroundFetchRegistration",
            "abort",
            result,
        );
        return;
    }
    let value = v8::Boolean::new(scope, true);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn match_record(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = data(scope, arguments.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "BackgroundFetchRegistration",
            "match",
            result,
        );
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'match' on 'BackgroundFetchRegistration': 1 argument required, but only 0 present.",
        );
        return;
    }
    let wanted = crate::webidl::value_to_string(scope, arguments.get(0));
    let found = record
        .records
        .iter()
        .find(|entry| {
            let entry = v8::Local::new(scope, *entry);
            super::background_fetch_record::request(scope, entry)
                .and_then(|request| super::request::url(scope, v8::Local::new(scope, &request)))
                .is_some_and(|url| url == wanted)
        })
        .cloned();
    let value = found
        .map(|entry| v8::Local::new(scope, &entry).into())
        .unwrap_or_else(|| v8::undefined(scope).into());
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

fn match_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = data(scope, arguments.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "BackgroundFetchRegistration",
            "matchAll",
            result,
        );
        return;
    };
    let values = v8::Array::new(scope, record.records.len() as i32);
    for (index, entry) in record.records.iter().enumerate() {
        let value = v8::Local::new(scope, entry);
        let _ = values.set_index(scope, index as u32, value.into());
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, values.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<BackgroundFetchRegistrationStore>() {
        store.constructor.remove(realm_id);
    }
}
