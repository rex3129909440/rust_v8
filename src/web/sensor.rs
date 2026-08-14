use std::collections::HashMap;

#[derive(Clone, Default)]
struct SensorRecord {
    active: bool,
    has_reading: bool,
    timestamp: Option<f64>,
    onerror: Option<v8::Global<v8::Value>>,
    onreading: Option<v8::Global<v8::Value>>,
    onactivate: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct SensorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SensorRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SensorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Sensor", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SensorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Sensor",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "activated", get_activated)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "hasReading", get_has_reading)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timestamp", get_timestamp)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_accessor(scope, prototype, "onreading", get_onreading, set_onreading)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onactivate",
        get_onactivate,
        set_onactivate,
    )?;
    crate::webidl::define_method(scope, prototype, "start", 0, start)?;
    crate::webidl::define_method(scope, prototype, "stop", 0, stop)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SensorStore>()
        .ok_or_else(|| "Sensor state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Sensor': Illegal constructor");
}

pub(crate) fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<SensorStore>()
        .expect("Sensor state")
        .records
        .insert(object.get_identity_hash().get(), SensorRecord::default());
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<SensorRecord> {
    scope
        .get_slot::<SensorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut SensorRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<SensorStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_activated(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.active).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_has_reading(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.has_reading).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_timestamp(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()).and_then(|record| record.timestamp) {
        Some(timestamp) => result.set(v8::Number::new(scope, timestamp).into()),
        None if record(scope, arguments.this()).is_some() => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn handler_get(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    field: impl FnOnce(SensorRecord) -> Option<v8::Global<v8::Value>>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, field(record), result);
}

fn get_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    handler_get(scope, arguments.this(), |record| record.onerror, result);
}

fn get_onreading(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    handler_get(scope, arguments.this(), |record| record.onreading, result);
}

fn get_onactivate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    handler_get(scope, arguments.this(), |record| record.onactivate, result);
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    field: impl FnOnce(&mut SensorRecord, Option<v8::Global<v8::Value>>),
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    update(scope, arguments.this(), |record| field(record, handler));
}

fn set_onerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(scope, arguments, |record, handler| record.onerror = handler);
}

fn set_onreading(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(scope, arguments, |record, handler| {
        record.onreading = handler
    });
}

fn set_onactivate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(scope, arguments, |record, handler| {
        record.onactivate = handler
    });
}

fn start(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.active {
        return;
    }
    if !crate::fingerprint::edge(scope).sensors.available {
        update(scope, arguments.this(), |record| {
            record.active = false;
            record.has_reading = false;
            record.timestamp = None;
        });
        let refreshed = record(scope, arguments.this()).expect("Sensor state");
        invoke_unavailable_error(scope, arguments.this(), refreshed.onerror);
        return;
    }
    let timestamp = super::performance::now_for_current_realm(scope).unwrap_or_else(|| {
        crate::determinism::relative_high_resolution_milliseconds(
            scope,
            crate::determinism::elapsed_milliseconds(scope),
            0.0,
        )
    });
    update(scope, arguments.this(), |record| {
        record.active = true;
        record.has_reading = true;
        record.timestamp = Some(timestamp);
    });
    let refreshed = record(scope, arguments.this()).expect("Sensor state");
    invoke(scope, arguments.this(), refreshed.onactivate, "activate");
    invoke(scope, arguments.this(), refreshed.onreading, "reading");
}

fn invoke_unavailable_error(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
) {
    let Some(handler) = handler else {
        return;
    };
    let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) else {
        return;
    };
    let Ok(error) = super::dom_exception::create(
        scope,
        "Could not connect to a sensor".to_owned(),
        "NotReadableError".to_owned(),
    ) else {
        return;
    };
    let Ok(event) = super::sensor_error_event::create(scope, error) else {
        return;
    };
    let _ = handler.call(scope, target.into(), &[event.into()]);
}

fn stop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| {
        record.active = false;
        record.has_reading = false;
        record.timestamp = None;
    });
}

fn invoke(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    handler: Option<v8::Global<v8::Value>>,
    event_type: &str,
) {
    let Some(handler) = handler else {
        return;
    };
    let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) else {
        return;
    };
    let Ok(event) = super::event::create(scope, event_type) else {
        return;
    };
    let _ = handler.call(scope, target.into(), &[event.into()]);
}
