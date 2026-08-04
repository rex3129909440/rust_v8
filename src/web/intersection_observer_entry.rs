use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct IntersectionObserverEntryStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, EntryRecord>,
}

#[derive(Clone)]
struct EntryRecord {
    time: f64,
    root_bounds: Option<v8::Global<v8::Object>>,
    bounding_client_rect: v8::Global<v8::Object>,
    intersection_rect: v8::Global<v8::Object>,
    is_intersecting: bool,
    is_visible: bool,
    intersection_ratio: f64,
    target: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IntersectionObserverEntryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IntersectionObserverEntry", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<IntersectionObserverEntryStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IntersectionObserverEntry",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "time", get_time)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rootBounds", get_root_bounds)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "boundingClientRect",
        get_bounding_client_rect,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "intersectionRect",
        get_intersection_rect,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "isIntersecting",
        get_is_intersecting,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "isVisible", get_is_visible)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "intersectionRatio",
        get_intersection_ratio,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "target", get_target)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IntersectionObserverEntryStore>()
        .ok_or_else(|| "IntersectionObserverEntry state was not prepared".to_owned())?
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
        "Failed to construct 'IntersectionObserverEntry': Illegal constructor",
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    time: f64,
    root_bounds: Option<v8::Local<'s, v8::Object>>,
    bounding_client_rect: v8::Local<'s, v8::Object>,
    intersection_rect: v8::Local<'s, v8::Object>,
    is_intersecting: bool,
    is_visible: bool,
    intersection_ratio: f64,
    target: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create IntersectionObserverEntry".to_owned());
    }
    let root_bounds = root_bounds.map(|value| v8::Global::new(scope, value));
    let bounding_client_rect = v8::Global::new(scope, bounding_client_rect);
    let intersection_rect = v8::Global::new(scope, intersection_rect);
    let target = v8::Global::new(scope, target);
    scope
        .get_slot_mut::<IntersectionObserverEntryStore>()
        .ok_or_else(|| "IntersectionObserverEntry state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            EntryRecord {
                time,
                root_bounds,
                bounding_client_rect,
                intersection_rect,
                is_intersecting,
                is_visible,
                intersection_ratio,
                target,
            },
        );
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<EntryRecord> {
    scope
        .get_slot::<IntersectionObserverEntryStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&EntryRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.time);
}
fn get_intersection_ratio(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.intersection_ratio);
}

fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&EntryRecord) -> Option<&v8::Global<v8::Object>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_root_bounds(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| x.root_bounds.as_ref());
}
fn get_bounding_client_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| Some(&x.bounding_client_rect));
}
fn get_intersection_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| Some(&x.intersection_rect));
}
fn get_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |x| Some(&x.target));
}

fn return_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&EntryRecord) -> bool,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_is_intersecting(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.is_intersecting);
}
fn get_is_visible(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_boolean(s, a, r, |x| x.is_visible);
}
