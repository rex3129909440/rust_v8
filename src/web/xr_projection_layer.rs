use std::collections::HashMap;

#[derive(Clone)]
struct ProjectionLayerRecord {
    texture_width: u32,
    texture_height: u32,
    texture_array_length: u32,
    ignore_depth_values: bool,
    fixed_foveation: f64,
    delta_pose: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct XrProjectionLayerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ProjectionLayerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrProjectionLayerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRProjectionLayer", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrProjectionLayerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRProjectionLayer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "textureWidth", get_texture_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "textureHeight", get_texture_height)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "textureArrayLength",
        get_texture_array_length,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "ignoreDepthValues",
        get_ignore_depth_values,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "fixedFoveation",
        get_fixed_foveation,
        set_fixed_foveation,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "deltaPose",
        get_delta_pose,
        set_delta_pose,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::xr_composition_layer::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrProjectionLayerStore>()
        .ok_or_else(|| "XRProjectionLayer state missing".to_owned())?
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

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRProjectionLayer".to_owned());
    }
    super::xr_composition_layer::attach(scope, object, options);
    let texture_width = options
        .map(|options| {
            super::event::number_property(scope, options, "textureWidth", 1280.0).max(1.0) as u32
        })
        .unwrap_or(1280);
    let texture_height = options
        .map(|options| {
            super::event::number_property(scope, options, "textureHeight", 720.0).max(1.0) as u32
        })
        .unwrap_or(720);
    let texture_array_length = options
        .map(|options| {
            super::event::number_property(scope, options, "textureArrayLength", 1.0).max(1.0) as u32
        })
        .unwrap_or(1);
    let ignore_depth_values = options
        .is_some_and(|options| super::event::boolean_property(scope, options, "ignoreDepthValues"));
    scope
        .get_slot_mut::<XrProjectionLayerStore>()
        .ok_or_else(|| "XRProjectionLayer state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ProjectionLayerRecord {
                texture_width,
                texture_height,
                texture_array_length,
                ignore_depth_values,
                fixed_foveation: 0.0,
                delta_pose: None,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ProjectionLayerRecord> {
    scope
        .get_slot::<XrProjectionLayerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_texture_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, state.texture_width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_texture_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, state.texture_height).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_texture_array_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, state.texture_array_length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_ignore_depth_values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, state.ignore_depth_values).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_fixed_foveation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, state.fixed_foveation).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_fixed_foveation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments
        .get(0)
        .number_value(scope)
        .unwrap_or(f64::NAN)
        .clamp(0.0, 1.0);
    if let Some(state) = scope
        .get_slot_mut::<XrProjectionLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.fixed_foveation = value;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_delta_pose(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(state) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = state.delta_pose {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_delta_pose(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments.get(0);
    let value = (!value.is_null() && !value.is_undefined()).then(|| v8::Global::new(scope, value));
    if let Some(state) = scope
        .get_slot_mut::<XrProjectionLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.delta_pose = value;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
