use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AudioParamStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
struct Record {
    value: f32,
    automation_rate: String,
    default_value: f32,
    min_value: f32,
    max_value: f32,
    context_identity: Option<i32>,
    events: Vec<AutomationEvent>,
}

#[derive(Clone)]
enum AutomationEvent {
    SetValue {
        time: f64,
        value: f32,
    },
    LinearRamp {
        time: f64,
        value: f32,
    },
    ExponentialRamp {
        time: f64,
        value: f32,
    },
    SetTarget {
        time: f64,
        target: f32,
        time_constant: f64,
    },
    ValueCurve {
        time: f64,
        duration: f64,
        values: Vec<f32>,
    },
}

impl AutomationEvent {
    fn time(&self) -> f64 {
        match self {
            Self::SetValue { time, .. }
            | Self::LinearRamp { time, .. }
            | Self::ExponentialRamp { time, .. }
            | Self::SetTarget { time, .. }
            | Self::ValueCurve { time, .. } => *time,
        }
    }
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioParamStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioParam", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AudioParamStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "AudioParam",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_accessor(scope, p, "value", get_value, set_value)?;
    crate::webidl::define_accessor(
        scope,
        p,
        "automationRate",
        get_automation_rate,
        set_automation_rate,
    )?;
    crate::webidl::define_readonly_accessor(scope, p, "defaultValue", get_default_value)?;
    crate::webidl::define_readonly_accessor(scope, p, "minValue", get_min_value)?;
    crate::webidl::define_readonly_accessor(scope, p, "maxValue", get_max_value)?;
    crate::webidl::define_method(scope, p, "cancelAndHoldAtTime", 1, cancel_and_hold_at_time)?;
    crate::webidl::define_method(
        scope,
        p,
        "cancelScheduledValues",
        1,
        cancel_scheduled_values,
    )?;
    crate::webidl::define_method(
        scope,
        p,
        "exponentialRampToValueAtTime",
        2,
        exponential_ramp_to_value_at_time,
    )?;
    crate::webidl::define_method(
        scope,
        p,
        "linearRampToValueAtTime",
        2,
        linear_ramp_to_value_at_time,
    )?;
    crate::webidl::define_method(scope, p, "setTargetAtTime", 3, set_target_at_time)?;
    crate::webidl::define_method(scope, p, "setValueAtTime", 2, set_value_at_time)?;
    crate::webidl::define_method(scope, p, "setValueCurveAtTime", 3, set_value_curve_at_time)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<AudioParamStore>()
        .ok_or_else(|| "AudioParam state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    default_value: f32,
    min_value: f32,
    max_value: f32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create AudioParam".to_owned());
    }
    scope
        .get_slot_mut::<AudioParamStore>()
        .ok_or_else(|| "AudioParam state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            Record {
                value: default_value,
                automation_rate: "a-rate".to_owned(),
                default_value,
                min_value,
                max_value,
                context_identity: Some(context.get_identity_hash().get()),
                events: Vec::new(),
            },
        );
    Ok(o)
}

pub(crate) fn is_param(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<AudioParamStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn associate_context(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<AudioParamStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.context_identity = Some(context.get_identity_hash().get());
        true
    } else {
        false
    }
}

pub(crate) fn context_identity(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    record(scope, object)?.context_identity
}

pub(crate) fn value_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<f32> {
    record(scope, object).map(|record| value_at_time(&record, time))
}

pub(crate) fn set_current_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: f32,
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<AudioParamStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.value = value.clamp(record.min_value, record.max_value);
        true
    } else {
        false
    }
}

pub(crate) fn set_initial_automation_rate(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    automation_rate: &str,
) -> bool {
    if automation_rate != "a-rate" && automation_rate != "k-rate" {
        return false;
    }
    if let Some(record) = scope
        .get_slot_mut::<AudioParamStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.automation_rate = automation_rate.to_owned();
        true
    } else {
        false
    }
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'AudioParam': Illegal constructor",
    );
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<AudioParamStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn update(
    scope: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) -> bool {
    if let Some(v) = scope
        .get_slot_mut::<AudioParamStore>()
        .and_then(|s| s.records.get_mut(&o.get_identity_hash().get()))
    {
        change(v);
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}
fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, v.value as f64).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(scope).unwrap_or(f64::NAN) as f32;
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "The provided float value is non-finite");
        return;
    }
    update(scope, a.this(), |v| {
        v.value = value.clamp(v.min_value, v.max_value)
    });
}
fn get_automation_rate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, &v.automation_rate) {
        r.set(s.into())
    }
}
fn set_automation_rate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if !matches!(value.as_str(), "a-rate" | "k-rate") {
        crate::webidl::throw_type_error(scope, "Invalid automationRate");
        return;
    }
    update(scope, a.this(), |v| v.automation_rate = value);
}
fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> f32,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Number::new(scope, select(&v) as f64).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_default_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.default_value)
}
fn get_min_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.min_value)
}
fn get_max_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.max_value)
}
fn valid_time(scope: &mut v8::PinScope<'_, '_>, time: f64) -> bool {
    if !time.is_finite() {
        crate::webidl::throw_type_error(scope, "The provided time must be finite and non-negative");
        return false;
    }
    if time < 0.0 {
        throw_range_error(scope, "Time must be a finite non-negative number");
        return false;
    }
    true
}

fn schedule(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    event: AutomationEvent,
) {
    if !valid_time(scope, event.time()) {
        return;
    }
    if update(scope, a.this(), |v| {
        v.events.push(event);
        v.events.sort_by(|x, y| x.time().total_cmp(&y.time()));
    }) {
        r.set(a.this().into())
    }
}
fn cancel_and_hold_at_time(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let time = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !valid_time(scope, time) {
        return;
    }
    let Some(snapshot) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let held = value_at_time(&snapshot, time);
    if update(scope, a.this(), |v| {
        v.events.retain(|event| event.time() < time);
        v.events
            .push(AutomationEvent::SetValue { time, value: held });
        v.events.sort_by(|x, y| x.time().total_cmp(&y.time()));
    }) {
        r.set(a.this().into())
    }
}
fn cancel_scheduled_values(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let time = a.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !valid_time(scope, time) {
        return;
    }
    if update(scope, a.this(), |v| {
        v.events.retain(|event| event.time() < time)
    }) {
        r.set(a.this().into())
    }
}
fn exponential_ramp_to_value_at_time(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(scope).unwrap_or(f64::NAN) as f32;
    let time = a.get(1).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "The provided float value is non-finite");
        return;
    }
    if value.abs() < f32::MIN_POSITIVE {
        throw_range_error(scope, "The exponential ramp target value must not be zero");
        return;
    }
    schedule(
        scope,
        a,
        r,
        AutomationEvent::ExponentialRamp { time, value },
    )
}
fn linear_ramp_to_value_at_time(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(scope).unwrap_or(f64::NAN) as f32;
    let time = a.get(1).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "The provided value must be finite");
        return;
    }
    schedule(scope, a, r, AutomationEvent::LinearRamp { time, value })
}
fn set_target_at_time(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(scope).unwrap_or(f64::NAN) as f32;
    let time = a.get(1).number_value(scope).unwrap_or(f64::NAN);
    let constant = a.get(2).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() || !constant.is_finite() || constant < 0.0 {
        crate::webidl::throw_type_error(
            scope,
            "The target and timeConstant must be finite and timeConstant non-negative",
        );
        return;
    }
    schedule(
        scope,
        a,
        r,
        AutomationEvent::SetTarget {
            time,
            target: value,
            time_constant: constant,
        },
    )
}
fn set_value_at_time(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = a.get(0).number_value(scope).unwrap_or(f64::NAN) as f32;
    let time = a.get(1).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "The provided value must be finite");
        return;
    }
    schedule(scope, a, r, AutomationEvent::SetValue { time, value })
}
fn set_value_curve_at_time(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "curve must be a typed array");
        return;
    };
    let time = a.get(1).number_value(scope).unwrap_or(f64::NAN);
    let duration = a.get(2).number_value(scope).unwrap_or(f64::NAN);
    if !valid_time(scope, time) {
        return;
    }
    if view.byte_length() < std::mem::size_of::<f32>() * 2 {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The curve length is less than the minimum bound (2)".to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    if !duration.is_finite() {
        crate::webidl::throw_type_error(scope, "The provided duration is non-finite");
        return;
    }
    if duration <= 0.0 {
        throw_range_error(scope, "Duration must be a finite positive number");
        return;
    }
    let object: v8::Local<v8::Object> = view.into();
    let length_key = v8::String::new(scope, "length").expect("length key");
    let length = object
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    let mut values = Vec::with_capacity(length as usize);
    for index in 0..length {
        let value = object
            .get_index(scope, index)
            .and_then(|value| value.number_value(scope))
            .unwrap_or(0.0) as f32;
        if !value.is_finite() {
            crate::webidl::throw_type_error(scope, "Curve values must be finite");
            return;
        }
        values.push(value);
    }
    schedule(
        scope,
        a,
        r,
        AutomationEvent::ValueCurve {
            time,
            duration,
            values,
        },
    )
}

fn throw_range_error(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Some(message) = v8::String::new(scope, message) {
        scope.throw_exception(v8::Exception::range_error(scope, message));
    }
}

fn value_at_time(record: &Record, time: f64) -> f32 {
    let mut value = record.value;
    let mut previous_time = 0.0;
    for event in &record.events {
        match event {
            AutomationEvent::SetValue {
                time: event_time,
                value: event_value,
            } => {
                if time < *event_time {
                    break;
                }
                value = *event_value;
                previous_time = *event_time;
            }
            AutomationEvent::LinearRamp {
                time: event_time,
                value: event_value,
            } => {
                if time < *event_time {
                    let span = (*event_time - previous_time).max(f64::EPSILON);
                    let amount = ((time - previous_time) / span).clamp(0.0, 1.0) as f32;
                    return value + (*event_value - value) * amount;
                }
                value = *event_value;
                previous_time = *event_time;
            }
            AutomationEvent::ExponentialRamp {
                time: event_time,
                value: event_value,
            } => {
                if time < *event_time {
                    if value * *event_value > 0.0 {
                        let span = (*event_time - previous_time).max(f64::EPSILON);
                        let amount = ((time - previous_time) / span).clamp(0.0, 1.0);
                        return (f64::from(value).signum()
                            * (f64::from(value).abs()
                                * (f64::from(*event_value).abs() / f64::from(value).abs())
                                    .powf(amount))) as f32;
                    }
                    return value;
                }
                value = *event_value;
                previous_time = *event_time;
            }
            AutomationEvent::SetTarget {
                time: event_time,
                target,
                time_constant,
            } => {
                if time < *event_time {
                    break;
                }
                value = if *time_constant == 0.0 {
                    *target
                } else {
                    let amount = (-(time - *event_time) / *time_constant).exp() as f32;
                    *target + (value - *target) * amount
                };
                previous_time = *event_time;
            }
            AutomationEvent::ValueCurve {
                time: event_time,
                duration,
                values,
            } => {
                if time < *event_time {
                    break;
                }
                if time >= *event_time + *duration {
                    value = *values.last().unwrap_or(&value);
                    previous_time = *event_time + *duration;
                    continue;
                }
                let position =
                    ((time - *event_time) / *duration) * (values.len().saturating_sub(1) as f64);
                let lower = position.floor() as usize;
                let upper = (lower + 1).min(values.len() - 1);
                let amount = (position - lower as f64) as f32;
                return values[lower] + (values[upper] - values[lower]) * amount;
            }
        }
    }
    value.clamp(record.min_value, record.max_value)
}
