use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WindowControlsOverlayStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, OverlayRecord>,
}

#[derive(Clone)]
struct OverlayRecord {
    visible: bool,
    rect: v8::Global<v8::Object>,
    on_geometry_change: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WindowControlsOverlayStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WindowControlsOverlay", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<WindowControlsOverlayStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "WindowControlsOverlay",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "visible", get_visible)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ongeometrychange",
        get_on_geometry_change,
        set_on_geometry_change,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getTitlebarAreaRect",
        0,
        get_titlebar_area_rect,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WindowControlsOverlayStore>()
        .ok_or_else(|| "WindowControlsOverlay state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let overlay = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, overlay, prototype.into()) != Some(true) {
        return Err("cannot create WindowControlsOverlay".to_owned());
    }
    super::event_target::attach(scope, overlay);
    let empty = v8::Object::new(scope);
    let rect = super::window_controls_overlay_geometry_change_event::copy_rect(scope, empty);
    let rect = v8::Global::new(scope, rect);
    scope
        .get_slot_mut::<WindowControlsOverlayStore>()
        .ok_or_else(|| "WindowControlsOverlay state was not prepared".to_owned())?
        .records
        .insert(
            overlay.get_identity_hash().get(),
            OverlayRecord {
                visible: false,
                rect,
                on_geometry_change: None,
            },
        );
    Ok(overlay)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'WindowControlsOverlay': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<OverlayRecord> {
    scope
        .get_slot::<WindowControlsOverlayStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_visible(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.visible).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_geometry_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.on_geometry_change {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_on_geometry_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let value = value.is_function().then(|| v8::Global::new(scope, value));
    let id = arguments.this().get_identity_hash().get();
    if let Some(record) = scope
        .get_slot_mut::<WindowControlsOverlayStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.on_geometry_change = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_titlebar_area_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.rect).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
