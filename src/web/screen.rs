use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ScreenStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
struct Record {
    avail_width: i32,
    avail_height: i32,
    width: i32,
    height: i32,
    color_depth: i32,
    pixel_depth: i32,
    avail_left: i32,
    avail_top: i32,
    orientation: v8::Global<v8::Object>,
    on_change: Option<v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ScreenStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Screen", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ScreenStore>()
        .and_then(|s| s.constructors.get(&realm_id))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "Screen",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "availWidth", get_avail_width)?;
    crate::webidl::define_readonly_accessor(scope, p, "availHeight", get_avail_height)?;
    crate::webidl::define_readonly_accessor(scope, p, "width", get_width)?;
    crate::webidl::define_readonly_accessor(scope, p, "height", get_height)?;
    crate::webidl::define_readonly_accessor(scope, p, "colorDepth", get_color_depth)?;
    crate::webidl::define_readonly_accessor(scope, p, "pixelDepth", get_pixel_depth)?;
    crate::webidl::define_readonly_accessor(scope, p, "availLeft", get_avail_left)?;
    crate::webidl::define_readonly_accessor(scope, p, "availTop", get_avail_top)?;
    crate::webidl::define_readonly_accessor(scope, p, "orientation", get_orientation)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_accessor(scope, p, "onchange", get_on_change, set_on_change)?;
    crate::webidl::define_readonly_accessor(scope, p, "isExtended", get_is_extended)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<ScreenStore>()
        .ok_or_else(|| "Screen state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    profile: &crate::ScreenFingerprint,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create Screen".to_owned());
    }
    let orientation = super::screen_orientation::create(scope)?;
    super::event_target::attach(scope, o);
    let record = Record {
        avail_width: profile.avail_width,
        avail_height: profile.avail_height,
        width: profile.width,
        height: profile.height,
        color_depth: profile.color_depth,
        pixel_depth: profile.pixel_depth,
        avail_left: profile.avail_left,
        avail_top: profile.avail_top,
        orientation: v8::Global::new(scope, orientation),
        on_change: None,
    };
    scope
        .get_slot_mut::<ScreenStore>()
        .ok_or_else(|| "Screen state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(o)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Screen': Illegal constructor");
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<ScreenStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn num(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> i32,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new(scope, select(&v)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_avail_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.avail_width)
}
fn get_avail_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.avail_height)
}
fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.width)
}
fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.height)
}
fn get_color_depth(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.color_depth)
}
fn get_pixel_depth(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.pixel_depth)
}
fn get_avail_left(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.avail_left)
}
fn get_avail_top(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    num(s, a, r, |v| v.avail_top)
}
fn get_orientation(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.orientation).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_on_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(screen) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, screen.on_change, result);
}

fn set_on_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(screen) = scope
        .get_slot_mut::<ScreenStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        screen.on_change = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_is_extended(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, false).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
