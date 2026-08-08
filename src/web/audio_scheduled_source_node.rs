use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AudioScheduledSourceNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ScheduledSourceRecord>,
}

#[derive(Clone, Default)]
struct ScheduledSourceRecord {
    object: Option<v8::Global<v8::Object>>,
    onended: Option<v8::Global<v8::Value>>,
    started_at: Option<f64>,
    stopped_at: Option<f64>,
    natural_end_at: Option<f64>,
    ended: bool,
}

pub(crate) enum StartSourceError {
    IllegalInvocation,
    AlreadyStarted,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioScheduledSourceNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioScheduledSourceNode", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AudioScheduledSourceNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioScheduledSourceNode",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "onended", get_onended, set_onended)?;
    crate::webidl::define_method(scope, prototype, "start", 0, start)?;
    crate::webidl::define_method(scope, prototype, "stop", 0, stop)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioScheduledSourceNodeStore>()
        .ok_or_else(|| "AudioScheduledSourceNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let stored = v8::Global::new(scope, object);
    if let Some(store) = scope.get_slot_mut::<AudioScheduledSourceNodeStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            ScheduledSourceRecord {
                object: Some(stored),
                ..ScheduledSourceRecord::default()
            },
        );
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'AudioScheduledSourceNode': Illegal constructor",
    );
}

fn get_onended(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<AudioScheduledSourceNodeStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.onended.clone());
    match value {
        Some(Some(value)) => result.set(v8::Local::new(scope, &value)),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_onended(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    let present = value.is_some();
    if let Some(record) = scope
        .get_slot_mut::<AudioScheduledSourceNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.onended = value;
        super::event_target::set_attribute_handler(scope, arguments.this(), "ended", present);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "ended" {
        return;
    }
    let handler = scope
        .get_slot::<AudioScheduledSourceNodeStore>()
        .and_then(|store| store.records.get(&target.get_identity_hash().get()))
        .and_then(|record| record.onended.clone());
    let Some(handler) = handler else {
        return;
    };
    let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) else {
        return;
    };
    let _ = handler.call(scope, target.into(), &[event.into()]);
}

fn time_argument(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<f64> {
    let time = if arguments.get(0).is_undefined() {
        0.0
    } else {
        arguments.get(0).number_value(scope).unwrap_or(f64::NAN)
    };
    if !time.is_finite() {
        crate::webidl::throw_type_error(
            scope,
            "The provided time must be a finite non-negative number",
        );
        return None;
    }
    if time < 0.0 {
        if let Some(message) = v8::String::new(
            scope,
            "The provided time is less than the minimum bound (0)",
        ) {
            scope.throw_exception(v8::Exception::range_error(scope, message));
        }
        return None;
    }
    Some(time)
}

fn start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(time) = time_argument(scope, &arguments) else {
        return;
    };
    match mark_started(scope, arguments.this(), time) {
        Ok(()) => {}
        Err(StartSourceError::IllegalInvocation) => {
            crate::webidl::throw_type_error(scope, "Illegal invocation");
        }
        Err(StartSourceError::AlreadyStarted) => {
            if let Ok(exception) = super::dom_exception::create(
                scope,
                "The source has already been started".to_owned(),
                "InvalidStateError".to_owned(),
            ) {
                scope.throw_exception(exception.into());
            }
        }
    }
}

pub(crate) fn mark_started(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Result<(), StartSourceError> {
    let Some(record) = scope
        .get_slot_mut::<AudioScheduledSourceNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return Err(StartSourceError::IllegalInvocation);
    };
    if record.started_at.is_some() {
        return Err(StartSourceError::AlreadyStarted);
    }
    record.started_at = Some(time);
    Ok(())
}

pub(crate) fn set_natural_end(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: Option<f64>,
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<AudioScheduledSourceNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    record.natural_end_at = time;
    true
}

pub(crate) fn is_active_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> bool {
    let Some(record) = scope
        .get_slot::<AudioScheduledSourceNodeStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
    else {
        return false;
    };
    let Some(started_at) = record.started_at else {
        return false;
    };
    time >= started_at && end_time(record).is_none_or(|end| time < end)
}

pub(crate) fn started_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<f64> {
    scope
        .get_slot::<AudioScheduledSourceNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())?
        .started_at
}

fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(time) = time_argument(scope, &arguments) else {
        return;
    };
    let Some(record) = scope
        .get_slot_mut::<AudioScheduledSourceNodeStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.started_at.is_none() {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The source has not been started".to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    record.stopped_at = Some(time);
}

pub(crate) fn run_pending(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let ready: Vec<i32> = scope
        .get_slot::<AudioScheduledSourceNodeStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter_map(|(identity, record)| {
                    if record.ended || record.started_at.is_none() {
                        return None;
                    }
                    let object = v8::Local::new(scope, record.object.as_ref()?);
                    let context = super::audio_node::context(scope, object)?;
                    if super::base_audio_context::state(scope, context).as_deref()
                        != Some("running")
                    {
                        return None;
                    }
                    let now = super::base_audio_context::current_time(scope, context)?;
                    end_time(record)
                        .is_some_and(|due| now >= due)
                        .then_some(*identity)
                })
                .collect()
        })
        .unwrap_or_default();
    for identity in &ready {
        dispatch_ended(scope, *identity);
    }
    !ready.is_empty()
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    let now_ms = crate::determinism::elapsed_milliseconds(scope);
    scope
        .get_slot::<AudioScheduledSourceNodeStore>()?
        .records
        .values()
        .filter_map(|record| {
            if record.ended || record.started_at.is_none() {
                return None;
            }
            let object = v8::Local::new(scope, record.object.as_ref()?);
            let context = super::audio_node::context(scope, object)?;
            if super::base_audio_context::state(scope, context).as_deref() != Some("running") {
                return None;
            }
            let current = super::base_audio_context::current_time(scope, context)?;
            let due = end_time(record)?;
            Some(now_ms + (due - current).max(0.0) * 1_000.0)
        })
        .min_by(f64::total_cmp)
}

fn end_time(record: &ScheduledSourceRecord) -> Option<f64> {
    match (record.stopped_at, record.natural_end_at) {
        (Some(stopped), Some(natural)) => Some(stopped.min(natural)),
        (Some(stopped), None) => Some(stopped),
        (None, natural) => natural,
    }
}

fn dispatch_ended(scope: &mut v8::PinScope<'_, '_>, identity: i32) {
    let snapshot = {
        let Some(record) = scope
            .get_slot_mut::<AudioScheduledSourceNodeStore>()
            .and_then(|store| store.records.get_mut(&identity))
        else {
            return;
        };
        if record.ended {
            return;
        }
        record.ended = true;
        record.clone()
    };
    let Some(object) = snapshot.object else {
        return;
    };
    let object = v8::Local::new(scope, &object);
    let Ok(event) = super::event::create(scope, "ended") else {
        return;
    };
    super::event_target::dispatch(scope, object, event);
}
