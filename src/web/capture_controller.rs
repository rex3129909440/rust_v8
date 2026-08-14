use std::collections::HashMap;

#[derive(Clone)]
struct CaptureRecord {
    focus_behavior: String,
    zoom_level: f64,
    handler: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct CaptureControllerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CaptureRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CaptureControllerStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(scope)?;
    crate::webidl::define_global(scope, "CaptureController", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<CaptureControllerStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "CaptureController",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_method(s, p, "setFocusBehavior", 1, set_focus_behavior)?;
    crate::webidl::define_readonly_accessor(s, p, "zoomLevel", zoom_level)?;
    crate::webidl::define_accessor(s, p, "onzoomlevelchange", get_handler, set_handler)?;
    crate::webidl::define_method(s, p, "decreaseZoomLevel", 0, decrease_zoom)?;
    crate::webidl::define_method(s, p, "forwardWheel", 1, forward_wheel)?;
    crate::webidl::define_method(s, p, "getSupportedZoomLevels", 0, get_supported_zoom_levels)?;
    crate::webidl::define_method(s, p, "increaseZoomLevel", 0, increase_zoom)?;
    crate::webidl::define_method(s, p, "resetZoomLevel", 0, reset_zoom)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<CaptureControllerStore>()
        .ok_or_else(|| "CaptureController state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "constructor must be called with new");
        return;
    }
    super::event_target::attach(s, a.this());
    s.get_slot_mut::<CaptureControllerStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            CaptureRecord {
                focus_behavior: "focus-capturing-application".to_owned(),
                zoom_level: 1.0,
                handler: None,
            },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<CaptureRecord> {
    s.get_slot::<CaptureControllerStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn resolve_void(s: &mut v8::PinScope<'_, '_>, mut r: v8::ReturnValue<'_>) {
    let v = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, v.into()) {
        r.set(p.into())
    }
}
fn set_focus_behavior(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let focus = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<CaptureControllerStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.focus_behavior = focus;
        resolve_void(s, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn zoom_level(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Number::new(s, v.zoom_level).into())
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
        .get_slot_mut::<CaptureControllerStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.handler = h
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn change_zoom(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    delta: f64,
    reset: bool,
    method_name: &str,
) {
    if let Some(v) = s
        .get_slot_mut::<CaptureControllerStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.zoom_level = if reset {
            1.0
        } else {
            (v.zoom_level + delta).clamp(0.5, 2.0)
        };
        resolve_void(s, r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "CaptureController", method_name, r)
    }
}
fn decrease_zoom(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    change_zoom(s, a, r, -0.5, false, "decreaseZoomLevel")
}
fn increase_zoom(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    change_zoom(s, a, r, 0.5, false, "increaseZoomLevel")
}
fn reset_zoom(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    change_zoom(s, a, r, 0.0, true, "resetZoomLevel")
}
fn forward_wheel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        resolve_void(s, r)
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "CaptureController", "forwardWheel", r)
    }
}
fn get_supported_zoom_levels(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let array = v8::Array::new(s, 4);
    let _ = array.set_index(s, 0, v8::Number::new(s, 0.5).into());
    let _ = array.set_index(s, 1, v8::Number::new(s, 1.0).into());
    let _ = array.set_index(s, 2, v8::Number::new(s, 1.5).into());
    let _ = array.set_index(s, 3, v8::Number::new(s, 2.0).into());
    r.set(array.into())
}
