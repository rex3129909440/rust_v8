use std::collections::HashMap;
#[derive(Clone)]
struct Details {
    screens: v8::Global<v8::Array>,
    current: v8::Global<v8::Object>,
    on_screens: Option<v8::Global<v8::Value>>,
    on_current: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct ScreenDetailsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Details>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(ScreenDetailsStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "ScreenDetails", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<ScreenDetailsStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "ScreenDetails",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "screens", screens)?;
    crate::webidl::define_readonly_accessor(s, p, "currentScreen", current)?;
    crate::webidl::define_accessor(s, p, "onscreenschange", get_screens, set_screens)?;
    crate::webidl::define_accessor(s, p, "oncurrentscreenchange", get_current, set_current)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let g = v8::Global::new(s, c);
    let realm_id = crate::webidl::realm_id(s);
    s.get_slot_mut::<ScreenDetailsStore>()
        .ok_or_else(|| "ScreenDetails state missing".to_owned())?
        .constructor
        .insert(realm_id, g);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create ScreenDetails".to_owned());
    }
    super::event_target::attach(s, o);
    let screen = super::screen_detailed::create(s)?;
    let array = v8::Array::new(s, 1);
    let _ = array.set_index(s, 0, screen.into());
    let d = Details {
        screens: v8::Global::new(s, array),
        current: v8::Global::new(s, screen),
        on_screens: None,
        on_current: None,
    };
    s.get_slot_mut::<ScreenDetailsStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), d);
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Details> {
    s.get_slot::<ScreenDetailsStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn screens(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.screens).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn current(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.current).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_screens(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.on_screens, r)
}
fn get_current(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.on_current, r)
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    screens: bool,
) {
    let h = super::window_event_handler_support::handler_value(s, value);
    if let Some(v) = s
        .get_slot_mut::<ScreenDetailsStore>()
        .and_then(|x| x.records.get_mut(&o.get_identity_hash().get()))
    {
        if screens {
            v.on_screens = h
        } else {
            v.on_current = h
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_screens(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a.this(), a.get(0), true)
}
fn set_current(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a.this(), a.get(0), false)
}
