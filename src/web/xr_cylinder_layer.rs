use std::collections::HashMap;

#[derive(Clone)]
struct CylinderLayerRecord {
    space: v8::Global<v8::Value>,
    transform: v8::Global<v8::Value>,
    radius: f64,
    central_angle: f64,
    aspect_ratio: f64,
    on_redraw: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct XrCylinderLayerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CylinderLayerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrCylinderLayerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRCylinderLayer", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrCylinderLayerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRCylinderLayer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "space", get_space, set_space)?;
    crate::webidl::define_accessor(scope, prototype, "transform", get_transform, set_transform)?;
    crate::webidl::define_accessor(scope, prototype, "radius", get_radius, set_radius)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "centralAngle",
        get_central_angle,
        set_central_angle,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "aspectRatio",
        get_aspect_ratio,
        set_aspect_ratio,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onredraw", get_on_redraw, set_on_redraw)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::xr_composition_layer::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrCylinderLayerStore>()
        .ok_or_else(|| "XRCylinderLayer state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

fn option_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = options?.get(scope, key.into())?;
    (!value.is_null() && !value.is_undefined()).then_some(value)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRCylinderLayer".to_owned());
    }
    super::xr_composition_layer::attach(scope, object, options);
    let space = option_value(scope, options, "space")
        .map(|value| v8::Global::new(scope, value))
        .unwrap_or_else(|| {
            let value: v8::Local<'_, v8::Value> = super::xr_space::create(scope)
                .map(|value| value.into())
                .unwrap_or_else(|_| v8::null(scope).into());
            v8::Global::new(scope, value)
        });
    let transform = option_value(scope, options, "transform")
        .map(|value| v8::Global::new(scope, value))
        .unwrap_or_else(|| {
            let value: v8::Local<'_, v8::Value> = super::xr_rigid_transform::create(scope)
                .map(|value| value.into())
                .unwrap_or_else(|_| v8::null(scope).into());
            v8::Global::new(scope, value)
        });
    let radius = options
        .map(|options| super::event::number_property(scope, options, "radius", 2.0))
        .unwrap_or(2.0);
    let central_angle = options
        .map(|options| {
            super::event::number_property(scope, options, "centralAngle", std::f64::consts::PI)
        })
        .unwrap_or(std::f64::consts::PI);
    let aspect_ratio = options
        .map(|options| super::event::number_property(scope, options, "aspectRatio", 2.0))
        .unwrap_or(2.0);
    scope
        .get_slot_mut::<XrCylinderLayerStore>()
        .ok_or_else(|| "XRCylinderLayer state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CylinderLayerRecord {
                space,
                transform,
                radius,
                central_angle,
                aspect_ratio,
                on_redraw: None,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CylinderLayerRecord> {
    scope
        .get_slot::<XrCylinderLayerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &state.space));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = v8::Global::new(scope, arguments.get(0));
    if let Some(state) = scope
        .get_slot_mut::<XrCylinderLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.space = value;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &state.transform));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = v8::Global::new(scope, arguments.get(0));
    if let Some(state) = scope
        .get_slot_mut::<XrCylinderLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.transform = value;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_radius(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    number_result(scope, arguments, result, |state| state.radius)
}

fn set_radius(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    set_number(scope, arguments, result, |state, value| {
        state.radius = value.max(0.0)
    })
}

fn get_central_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    number_result(scope, arguments, result, |state| state.central_angle)
}

fn set_central_angle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    set_number(scope, arguments, result, |state, value| {
        state.central_angle = value
    })
}

fn get_aspect_ratio(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    number_result(scope, arguments, result, |state| state.aspect_ratio)
}

fn set_aspect_ratio(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    set_number(scope, arguments, result, |state, value| {
        state.aspect_ratio = value.max(0.0)
    })
}

fn number_result(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: fn(&CylinderLayerRecord) -> f64,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&state)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    assign: fn(&mut CylinderLayerRecord, f64),
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if let Some(state) = scope
        .get_slot_mut::<XrCylinderLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        assign(state, value);
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(state) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = state.on_redraw {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_on_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments.get(0);
    let handler = value.is_function().then(|| v8::Global::new(scope, value));
    if let Some(state) = scope
        .get_slot_mut::<XrCylinderLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.on_redraw = handler;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
