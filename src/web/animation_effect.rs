use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AnimationEffectStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TimingRecord>,
}

#[derive(Clone)]
pub(crate) struct TimingRecord {
    delay: f64,
    direction: String,
    duration: Option<f64>,
    easing: String,
    end_delay: f64,
    fill: String,
    iteration_start: f64,
    iterations: f64,
}

impl Default for TimingRecord {
    fn default() -> Self {
        Self {
            delay: 0.0,
            direction: "normal".to_owned(),
            duration: None,
            easing: "linear".to_owned(),
            end_delay: 0.0,
            fill: "auto".to_owned(),
            iteration_start: 0.0,
            iterations: 1.0,
        }
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationEffectStore::default());
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AnimationEffect", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AnimationEffectStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AnimationEffect",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getComputedTiming",
        0,
        get_computed_timing,
    )?;
    crate::webidl::define_method(scope, prototype, "getTiming", 0, get_timing)?;
    crate::webidl::define_method(scope, prototype, "updateTiming", 0, update_timing)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AnimationEffectStore>()
        .ok_or_else(|| "AnimationEffect state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'AnimationEffect': Illegal constructor",
    );
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Value>>,
) {
    let mut timing = TimingRecord::default();
    if let Some(options) = options {
        apply_options(scope, &mut timing, options);
    }
    if let Some(store) = scope.get_slot_mut::<AnimationEffectStore>() {
        store
            .records
            .insert(object.get_identity_hash().get(), timing);
    }
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<TimingRecord> {
    scope
        .get_slot::<AnimationEffectStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_timing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(timing_object(scope, &record, false).into());
}

fn get_computed_timing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(timing_object(scope, &record, true).into());
}

fn update_timing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(mut record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    apply_options(scope, &mut record, arguments.get(0));
    if let Some(stored) = scope
        .get_slot_mut::<AnimationEffectStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *stored = record;
    }
}

fn apply_options(
    scope: &v8::PinScope<'_, '_>,
    timing: &mut TimingRecord,
    value: v8::Local<'_, v8::Value>,
) {
    if value.is_number() {
        timing.duration = value.number_value(scope);
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        return;
    };
    if let Some(value) = number_property(scope, options, "delay") {
        timing.delay = value;
    }
    if let Some(value) = string_property(scope, options, "direction") {
        timing.direction = value;
    }
    if let Some(value) = value_property(scope, options, "duration") {
        timing.duration =
            if value.is_string() && crate::webidl::value_to_string(scope, value) == "auto" {
                None
            } else {
                value.number_value(scope)
            };
    }
    if let Some(value) = string_property(scope, options, "easing") {
        timing.easing = value;
    }
    if let Some(value) = number_property(scope, options, "endDelay") {
        timing.end_delay = value;
    }
    if let Some(value) = string_property(scope, options, "fill") {
        timing.fill = value;
    }
    if let Some(value) = number_property(scope, options, "iterationStart") {
        timing.iteration_start = value;
    }
    if let Some(value) = number_property(scope, options, "iterations") {
        timing.iterations = value;
    }
}

fn timing_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    timing: &TimingRecord,
    computed: bool,
) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    define_number(scope, object, "delay", timing.delay);
    define_string(scope, object, "direction", &timing.direction);
    if let Some(duration) = timing.duration {
        define_number(scope, object, "duration", duration);
    } else if computed {
        define_number(scope, object, "duration", 0.0);
    } else {
        define_string(scope, object, "duration", "auto");
    }
    define_string(scope, object, "easing", &timing.easing);
    define_number(scope, object, "endDelay", timing.end_delay);
    define_string(
        scope,
        object,
        "fill",
        if computed && timing.fill == "auto" {
            "none"
        } else {
            &timing.fill
        },
    );
    define_number(scope, object, "iterationStart", timing.iteration_start);
    define_number(scope, object, "iterations", timing.iterations);
    if computed {
        let duration = timing.duration.unwrap_or(0.0);
        define_number(
            scope,
            object,
            "activeDuration",
            duration * timing.iterations,
        );
        define_null(scope, object, "currentIteration");
        define_number(
            scope,
            object,
            "endTime",
            timing.delay + duration * timing.iterations + timing.end_delay,
        );
        define_null(scope, object, "localTime");
        define_null(scope, object, "progress");
    }
    object
}

fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then_some(value)
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    value_property(scope, object, name).map(|value| crate::webidl::value_to_string(scope, value))
}

fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    value_property(scope, object, name).and_then(|value| value.number_value(scope))
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    define(scope, object, name, v8::Number::new(scope, value).into());
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define(scope, object, name, value.into());
    }
}

fn define_null(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, name: &str) {
    define(scope, object, name, v8::null(scope).into());
}

fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), value);
    }
}
