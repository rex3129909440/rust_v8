use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformancePaintTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PaintTimingRecord>,
}

#[derive(Clone)]
struct PaintTimingRecord {
    paint_time: f64,
    presentation_time: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformancePaintTimingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformancePaintTiming", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformancePaintTimingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformancePaintTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "paintTime", get_paint_time)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "presentationTime",
        get_presentation_time,
    )?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformancePaintTimingStore>()
        .ok_or_else(|| "PerformancePaintTiming state was not prepared".to_owned())?
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
        "Failed to construct 'PerformancePaintTiming': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    start_time: f64,
    paint_time: f64,
    presentation_time: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformancePaintTiming".to_owned());
    }
    super::performance_entry::attach(scope, timing, name, "paint".to_owned(), start_time, 0.0);
    scope
        .get_slot_mut::<PerformancePaintTimingStore>()
        .ok_or_else(|| "PerformancePaintTiming state was not prepared".to_owned())?
        .records
        .insert(
            timing.get_identity_hash().get(),
            PaintTimingRecord {
                paint_time,
                presentation_time,
            },
        );
    Ok(timing)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PaintTimingRecord> {
    scope
        .get_slot::<PerformancePaintTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_paint_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.paint_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_presentation_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.presentation_time).into());
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
    define_number(scope, output, "paintTime", record.paint_time);
    define_number(scope, output, "presentationTime", record.presentation_time);
    result.set(output.into());
}
