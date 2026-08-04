use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AnimationTimelineStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AnimationTimelineRecord>,
}

#[derive(Clone)]
struct AnimationTimelineRecord {
    current_time: Option<f64>,
    duration: Option<f64>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationTimelineStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AnimationTimeline", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AnimationTimelineStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AnimationTimeline",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "currentTime", get_current_time)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "duration", get_duration)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AnimationTimelineStore>()
        .ok_or_else(|| "AnimationTimeline state was not prepared".to_owned())?
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
        "Failed to construct 'AnimationTimeline': Illegal constructor",
    );
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    current_time: Option<f64>,
    duration: Option<f64>,
) {
    if let Some(store) = scope.get_slot_mut::<AnimationTimelineStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            AnimationTimelineRecord {
                current_time,
                duration,
            },
        );
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AnimationTimelineRecord> {
    scope
        .get_slot::<AnimationTimelineStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_optional_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AnimationTimelineRecord) -> Option<f64>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Number::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_current_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_optional_number(s, a, r, |record| record.current_time)
}
fn get_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_optional_number(s, a, r, |record| record.duration)
}
