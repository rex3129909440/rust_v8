use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct XmlHttpRequestEventTargetStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, HandlerRecord>,
}

#[derive(Clone, Copy)]
pub(crate) enum ProgressHandler {
    LoadStart,
    Progress,
    Abort,
    Error,
    Load,
    Timeout,
    LoadEnd,
}

#[derive(Clone, Default)]
pub(crate) struct HandlerRecord {
    pub(crate) load_start: Option<v8::Global<v8::Value>>,
    pub(crate) progress: Option<v8::Global<v8::Value>>,
    pub(crate) abort: Option<v8::Global<v8::Value>>,
    pub(crate) error: Option<v8::Global<v8::Value>>,
    pub(crate) load: Option<v8::Global<v8::Value>>,
    pub(crate) timeout: Option<v8::Global<v8::Value>>,
    pub(crate) load_end: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XmlHttpRequestEventTargetStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XMLHttpRequestEventTarget", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<XmlHttpRequestEventTargetStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "XMLHttpRequestEventTarget",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::xml_http_request_event_target_onloadstart_property::define(scope, prototype)?;
    super::xml_http_request_event_target_onprogress_property::define(scope, prototype)?;
    super::xml_http_request_event_target_onabort_property::define(scope, prototype)?;
    super::xml_http_request_event_target_onerror_property::define(scope, prototype)?;
    super::xml_http_request_event_target_onload_property::define(scope, prototype)?;
    super::xml_http_request_event_target_ontimeout_property::define(scope, prototype)?;
    super::xml_http_request_event_target_onloadend_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XmlHttpRequestEventTargetStore>()
        .ok_or_else(|| "XMLHttpRequestEventTarget state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    super::event_target::attach(scope, object);
    if let Some(store) = scope.get_slot_mut::<XmlHttpRequestEventTargetStore>() {
        store
            .records
            .entry(object.get_identity_hash().get())
            .or_default();
    }
}

pub(crate) fn fire(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event_name: &str,
    slot: ProgressHandler,
) {
    let event = super::event_target::create_event(scope, event_name);
    let handler = scope
        .get_slot::<XmlHttpRequestEventTargetStore>()
        .and_then(|store| store.records.get(&target.get_identity_hash().get()))
        .and_then(|record| select(record, slot));
    if let Some(handler) = handler {
        if let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) {
            let _ = handler.call(scope, target.into(), &[event.into()]);
        }
    }
    super::event_target::dispatch(scope, target, event);
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'XMLHttpRequestEventTarget': Illegal constructor",
    );
}

pub(crate) fn select(
    record: &HandlerRecord,
    slot: ProgressHandler,
) -> Option<v8::Global<v8::Value>> {
    match slot {
        ProgressHandler::LoadStart => record.load_start.clone(),
        ProgressHandler::Progress => record.progress.clone(),
        ProgressHandler::Abort => record.abort.clone(),
        ProgressHandler::Error => record.error.clone(),
        ProgressHandler::Load => record.load.clone(),
        ProgressHandler::Timeout => record.timeout.clone(),
        ProgressHandler::LoadEnd => record.load_end.clone(),
    }
}

pub(crate) fn assign(
    record: &mut HandlerRecord,
    slot: ProgressHandler,
    value: Option<v8::Global<v8::Value>>,
) {
    match slot {
        ProgressHandler::LoadStart => record.load_start = value,
        ProgressHandler::Progress => record.progress = value,
        ProgressHandler::Abort => record.abort = value,
        ProgressHandler::Error => record.error = value,
        ProgressHandler::Load => record.load = value,
        ProgressHandler::Timeout => record.timeout = value,
        ProgressHandler::LoadEnd => record.load_end = value,
    }
}

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    slot: ProgressHandler,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot::<XmlHttpRequestEventTargetStore>()
        .and_then(|store| store.records.get(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(record, slot) {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    slot: ProgressHandler,
) {
    let value = arguments.get(0);
    let value = value.is_function().then(|| v8::Global::new(scope, value));
    let id = arguments.this().get_identity_hash().get();
    let Some(record) = scope
        .get_slot_mut::<XmlHttpRequestEventTargetStore>()
        .and_then(|store| store.records.get_mut(&id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    assign(record, slot, value);
}

pub(crate) fn get_on_load_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ProgressHandler::LoadStart);
}
pub(crate) fn set_on_load_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ProgressHandler::LoadStart);
}
pub(crate) fn get_on_progress(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ProgressHandler::Progress);
}
pub(crate) fn set_on_progress(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ProgressHandler::Progress);
}
pub(crate) fn get_on_abort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ProgressHandler::Abort);
}
pub(crate) fn set_on_abort(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ProgressHandler::Abort);
}
pub(crate) fn get_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ProgressHandler::Error);
}
pub(crate) fn set_on_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ProgressHandler::Error);
}
pub(crate) fn get_on_load(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ProgressHandler::Load);
}
pub(crate) fn set_on_load(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ProgressHandler::Load);
}
pub(crate) fn get_on_timeout(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ProgressHandler::Timeout);
}
pub(crate) fn set_on_timeout(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ProgressHandler::Timeout);
}
pub(crate) fn get_on_load_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, ProgressHandler::LoadEnd);
}
pub(crate) fn set_on_load_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, ProgressHandler::LoadEnd);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<XmlHttpRequestEventTargetStore>() {
        store.constructor.remove(realm_id);
    }
}
