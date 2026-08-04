use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ScrollTimelineStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ScrollTimelineRecord>,
}

#[derive(Clone)]
struct ScrollTimelineRecord {
    source: Option<v8::Global<v8::Object>>,
    axis: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ScrollTimelineStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ScrollTimeline", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ScrollTimelineStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ScrollTimeline",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "source", get_source)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "axis", get_axis)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let animation_timeline = super::animation_timeline::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, animation_timeline)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ScrollTimelineStore>()
        .ok_or_else(|| "ScrollTimeline state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "ScrollTimeline must be constructed with new");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let source = options.and_then(|options| object_property(scope, options, "source"));
    let axis = options
        .and_then(|options| string_property(scope, options, "axis"))
        .unwrap_or_else(|| "block".to_owned());
    if axis != "block" && axis != "inline" && axis != "x" && axis != "y" {
        crate::webidl::throw_type_error(scope, "Invalid ScrollTimeline axis");
        return;
    }
    attach(scope, arguments.this(), source, axis);
    result.set(arguments.this().into());
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    source: Option<v8::Local<'_, v8::Object>>,
    axis: String,
) {
    super::animation_timeline::attach(scope, object, Some(0.0), Some(100.0));
    let source = source.map(|source| v8::Global::new(scope, source));
    if let Some(store) = scope.get_slot_mut::<ScrollTimelineStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            ScrollTimelineRecord { source, axis },
        );
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ScrollTimelineRecord> {
    scope
        .get_slot::<ScrollTimelineStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(source) = record.source {
        result.set(v8::Local::new(scope, &source).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_axis(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(axis) = v8::String::new(scope, &record.axis) {
            result.set(axis.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null() || value.is_undefined() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(value).ok()
    }
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then(|| crate::webidl::value_to_string(scope, value))
}
