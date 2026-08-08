use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct VisualViewportStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, VisualViewportRecord>,
}

#[derive(Clone)]
struct VisualViewportRecord {
    offset_left: f64,
    offset_top: f64,
    page_left: f64,
    page_top: f64,
    scale: f64,
    onresize: Option<v8::Global<v8::Value>>,
    onscroll: Option<v8::Global<v8::Value>>,
    onscrollend: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(VisualViewportStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "VisualViewport", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<VisualViewportStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "VisualViewport",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "offsetLeft", get_offset_left)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "offsetTop", get_offset_top)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pageLeft", get_page_left)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pageTop", get_page_top)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "width", get_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "height", get_height)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "scale", get_scale)?;
    crate::webidl::define_accessor(scope, prototype, "onresize", get_onresize, set_onresize)?;
    crate::webidl::define_accessor(scope, prototype, "onscroll", get_onscroll, set_onscroll)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onscrollend",
        get_onscrollend,
        set_onscrollend,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let event_target = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<VisualViewportStore>()
        .ok_or_else(|| "VisualViewport state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    profile: &crate::ScreenFingerprint,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create VisualViewport".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<VisualViewportStore>()
        .ok_or_else(|| "VisualViewport state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            VisualViewportRecord {
                offset_left: profile.visual_viewport_offset_left,
                offset_top: profile.visual_viewport_offset_top,
                page_left: profile.visual_viewport_page_left,
                page_top: profile.visual_viewport_page_top,
                scale: profile.visual_viewport_scale,
                onresize: None,
                onscroll: None,
                onscrollend: None,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<VisualViewportRecord> {
    scope
        .get_slot::<VisualViewportStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut VisualViewportRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<VisualViewportStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&VisualViewportRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_offset_left(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.offset_left)
}
fn get_offset_top(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.offset_top)
}
fn get_page_left(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.page_left)
}
fn get_page_top(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.page_top)
}
fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Number::new(s, super::window_view_state::inner_width(s)).into());
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}
fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Number::new(s, super::window_view_state::inner_height(s)).into());
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}
fn get_scale(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.scale)
}

fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&VisualViewportRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = select(&record) {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_onresize(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.onresize.clone())
}
fn get_onscroll(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.onscroll.clone())
}
fn get_onscrollend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.onscrollend.clone())
}

fn handler_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    if value.is_object() {
        Some(v8::Global::new(scope, value))
    } else {
        None
    }
}

fn set_onresize(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(s, a.get(0));
    update(s, a.this(), |record| record.onresize = value)
}
fn set_onscroll(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(s, a.get(0));
    update(s, a.this(), |record| record.onscroll = value)
}
fn set_onscrollend(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(s, a.get(0));
    update(s, a.this(), |record| record.onscrollend = value)
}
