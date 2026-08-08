use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WindowControlsOverlayGeometryChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, GeometryChangeRecord>,
}

#[derive(Clone)]
pub(crate) struct GeometryChangeRecord {
    pub(crate) titlebar_area_rect: Option<v8::Global<v8::Object>>,
    pub(crate) visible: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WindowControlsOverlayGeometryChangeEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "WindowControlsOverlayGeometryChangeEvent",
        constructor.into(),
    )
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<WindowControlsOverlayGeometryChangeEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "WindowControlsOverlayGeometryChangeEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::window_controls_overlay_geometry_change_event_titlebar_area_rect_property::define(
        scope, prototype,
    )?;
    super::window_controls_overlay_geometry_change_event_visible_property::define(
        scope, prototype,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WindowControlsOverlayGeometryChangeEventStore>()
        .ok_or_else(|| {
            "WindowControlsOverlayGeometryChangeEvent state was not prepared".to_owned()
        })?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WindowControlsOverlayGeometryChangeEvent': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WindowControlsOverlayGeometryChangeEvent': 2 arguments required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "The event initializer must be an object");
        return;
    };
    let visible = super::event::boolean_property(scope, init, "visible");
    let titlebar_area_rect = object_property(scope, init, "titlebarAreaRect")
        .map(|rect| v8::Global::new(scope, copy_rect(scope, rect)));
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<WindowControlsOverlayGeometryChangeEventStore>()
        .expect("WindowControlsOverlayGeometryChangeEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            GeometryChangeRecord {
                titlebar_area_rect,
                visible,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<GeometryChangeRecord> {
    scope
        .get_slot::<WindowControlsOverlayGeometryChangeEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_titlebar_area_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(rect) = record.titlebar_area_rect {
        result.set(v8::Local::new(scope, &rect).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_visible(
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

pub(crate) fn copy_rect<'s>(
    scope: &v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
) -> v8::Local<'s, v8::Object> {
    let x = super::event::number_property(scope, source, "x", 0.0);
    let y = super::event::number_property(scope, source, "y", 0.0);
    let width = super::event::number_property(scope, source, "width", 0.0);
    let height = super::event::number_property(scope, source, "height", 0.0);
    let rect = v8::Object::new(scope);
    define_number(scope, rect, "x", x);
    define_number(scope, rect, "y", y);
    define_number(scope, rect, "width", width);
    define_number(scope, rect, "height", height);
    define_number(scope, rect, "top", y.min(y + height));
    define_number(scope, rect, "right", x.max(x + width));
    define_number(scope, rect, "bottom", y.max(y + height));
    define_number(scope, rect, "left", x.min(x + width));
    rect
}

pub(crate) fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    v8::Local::<v8::Object>::try_from(value).ok()
}

pub(crate) fn define_number(
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
