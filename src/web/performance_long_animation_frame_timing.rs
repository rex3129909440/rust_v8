use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceLongAnimationFrameTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, LongAnimationFrameRecord>,
}

#[derive(Clone)]
struct LongAnimationFrameRecord {
    render_start: f64,
    style_and_layout_start: f64,
    first_ui_event_timestamp: f64,
    blocking_duration: f64,
    scripts: Vec<v8::Global<v8::Object>>,
    paint_time: f64,
    presentation_time: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceLongAnimationFrameTimingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "PerformanceLongAnimationFrameTiming",
        constructor.into(),
    )
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceLongAnimationFrameTimingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceLongAnimationFrameTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "renderStart", get_render_start)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "styleAndLayoutStart",
        get_style_and_layout_start,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "firstUIEventTimestamp",
        get_first_ui_event_timestamp,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "blockingDuration",
        get_blocking_duration,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "scripts", get_scripts)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "paintTime", get_paint_time)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "presentationTime",
        get_presentation_time,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceLongAnimationFrameTimingStore>()
        .ok_or_else(|| "PerformanceLongAnimationFrameTiming state was not prepared".to_owned())?
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
        "Failed to construct 'PerformanceLongAnimationFrameTiming': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    start_time: f64,
    duration: f64,
    scripts: Vec<v8::Global<v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceLongAnimationFrameTiming".to_owned());
    }
    super::performance_entry::attach(
        scope,
        timing,
        "long-animation-frame".to_owned(),
        "long-animation-frame".to_owned(),
        start_time,
        duration,
    );
    scope
        .get_slot_mut::<PerformanceLongAnimationFrameTimingStore>()
        .ok_or_else(|| "PerformanceLongAnimationFrameTiming state was not prepared".to_owned())?
        .records
        .insert(
            timing.get_identity_hash().get(),
            LongAnimationFrameRecord {
                render_start: start_time,
                style_and_layout_start: start_time,
                first_ui_event_timestamp: 0.0,
                blocking_duration: (duration - 50.0).max(0.0),
                scripts,
                paint_time: 0.0,
                presentation_time: 0.0,
            },
        );
    Ok(timing)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<LongAnimationFrameRecord> {
    scope
        .get_slot::<PerformanceLongAnimationFrameTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&LongAnimationFrameRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_render_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.render_start);
}
fn get_style_and_layout_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.style_and_layout_start);
}
fn get_first_ui_event_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.first_ui_event_timestamp);
}
fn get_blocking_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.blocking_duration);
}
fn get_paint_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.paint_time);
}
fn get_presentation_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.presentation_time);
}

fn scripts_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    values: &[v8::Global<v8::Object>],
) -> v8::Local<'s, v8::Array> {
    let output = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let _ = output.set_index(scope, index as u32, v8::Local::new(scope, value).into());
    }
    output
}

fn get_scripts(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(scripts_array(scope, &record.scripts).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
    }
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(base) = super::performance_entry::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = super::performance_entry::to_object(scope, &base);
    define_number(scope, output, "renderStart", record.render_start);
    define_number(
        scope,
        output,
        "styleAndLayoutStart",
        record.style_and_layout_start,
    );
    define_number(
        scope,
        output,
        "firstUIEventTimestamp",
        record.first_ui_event_timestamp,
    );
    define_number(scope, output, "blockingDuration", record.blocking_duration);
    if let Some(key) = v8::String::new(scope, "scripts") {
        let _ = output.create_data_property(
            scope,
            key.into(),
            scripts_array(scope, &record.scripts).into(),
        );
    }
    define_number(scope, output, "paintTime", record.paint_time);
    define_number(scope, output, "presentationTime", record.presentation_time);
    result.set(output.into());
}
