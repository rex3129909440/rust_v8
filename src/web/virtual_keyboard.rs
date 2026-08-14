use std::collections::HashMap;
#[derive(Clone, Default)]
struct KeyboardState {
    visible: bool,
    overlays: bool,
    handler: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct VirtualKeyboardStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, KeyboardState>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(VirtualKeyboardStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "VirtualKeyboard", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<VirtualKeyboardStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "VirtualKeyboard",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "boundingRect", rect)?;
    crate::webidl::define_accessor(s, p, "overlaysContent", get_overlays, set_overlays)?;
    crate::webidl::define_accessor(s, p, "ongeometrychange", get_handler, set_handler)?;
    crate::webidl::define_method(s, p, "hide", 0, hide)?;
    crate::webidl::define_method(s, p, "show", 0, show)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let g = v8::Global::new(s, c);
    let realm_id = crate::webidl::realm_id(s);
    s.get_slot_mut::<VirtualKeyboardStore>()
        .ok_or_else(|| "VirtualKeyboard state missing".to_owned())?
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
        return Err("cannot create VirtualKeyboard".to_owned());
    }
    super::event_target::attach(s, o);
    s.get_slot_mut::<VirtualKeyboardStore>()
        .unwrap()
        .records
        .insert(o.get_identity_hash().get(), KeyboardState::default());
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<KeyboardState> {
    s.get_slot::<VirtualKeyboardStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        let h = if v.visible { 300.0 } else { 0.0 };
        let bounds = super::dom_rect_read_only::RectRecord {
            x: 0.0,
            y: 800.0 - h,
            width: 1280.0,
            height: h,
        };
        if let Ok(rect) = super::dom_rect::create(s, bounds) {
            r.set(rect.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_overlays(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.overlays).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_overlays(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(s);
    if let Some(v) = s
        .get_slot_mut::<VirtualKeyboardStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.overlays = value
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.handler, r)
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<VirtualKeyboardStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn visible(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, visible: bool) {
    if let Some(v) = s
        .get_slot_mut::<VirtualKeyboardStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.visible = visible
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn show(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    visible(s, a, true)
}
fn hide(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    visible(s, a, false)
}
