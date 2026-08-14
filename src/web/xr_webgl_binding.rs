use std::collections::HashMap;

#[derive(Clone)]
struct BindingRecord {
    session: v8::Global<v8::Value>,
    context: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct XrWebGlBindingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, BindingRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrWebGlBindingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRWebGLBinding", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrWebGlBindingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRWebGLBinding",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "nativeProjectionScaleFactor",
        native_projection_scale_factor,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "usesDepthValues",
        uses_depth_values,
    )?;
    crate::webidl::define_method(scope, prototype, "createCubeLayer", 1, create_cube_layer)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createCylinderLayer",
        1,
        create_cylinder_layer,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createEquirectLayer",
        1,
        create_equirect_layer,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createProjectionLayer",
        0,
        create_projection_layer,
    )?;
    crate::webidl::define_method(scope, prototype, "createQuadLayer", 1, create_quad_layer)?;
    crate::webidl::define_method(scope, prototype, "getSubImage", 2, get_sub_image)?;
    crate::webidl::define_method(scope, prototype, "getViewSubImage", 2, get_view_sub_image)?;
    crate::webidl::define_method(scope, prototype, "getCameraImage", 1, get_camera_image)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getDepthInformation",
        1,
        get_depth_information,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getReflectionCubeMap",
        1,
        get_reflection_cube_map,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrWebGlBindingStore>()
        .ok_or_else(|| "XRWebGLBinding state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "2 arguments required");
        return;
    }
    if arguments.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'XRWebGLBinding': parameter 1 is not of type 'XRSession'.",
        );
        return;
    }
    let valid_session = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .is_some_and(|object| {
            super::structured_clone::inherits_platform_interface(scope, object, "XRSession")
        });
    if !valid_session {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'XRWebGLBinding': parameter 1 is not of type 'XRSession'.",
        );
        return;
    }
    let record = BindingRecord {
        session: v8::Global::new(scope, arguments.get(0)),
        context: v8::Global::new(scope, arguments.get(1)),
    };
    scope
        .get_slot_mut::<XrWebGlBindingStore>()
        .expect("XRWebGLBinding state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BindingRecord> {
    scope
        .get_slot::<XrWebGlBindingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn require(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BindingRecord> {
    let value = record(scope, object);
    if value.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    value
}

fn native_projection_scale_factor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require(scope, arguments.this()).is_some() {
        result.set(v8::Number::new(scope, 1.0).into());
    }
}

fn uses_depth_values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, true).into());
    }
}

fn options_argument<'s>(
    arguments: v8::FunctionCallbackArguments<'s>,
) -> Option<v8::Local<'s, v8::Object>> {
    v8::Local::<v8::Object>::try_from(arguments.get(0)).ok()
}

fn create_cube_layer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require(scope, arguments.this()).is_none() {
        return;
    }
    let options = options_argument(arguments);
    match super::xr_cube_layer::create(scope, options) {
        Ok(layer) => result.set(layer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_cylinder_layer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require(scope, arguments.this()).is_none() {
        return;
    }
    let options = options_argument(arguments);
    match super::xr_cylinder_layer::create(scope, options) {
        Ok(layer) => result.set(layer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_equirect_layer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require(scope, arguments.this()).is_none() {
        return;
    }
    let options = options_argument(arguments);
    match super::xr_equirect_layer::create(scope, options) {
        Ok(layer) => result.set(layer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_projection_layer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require(scope, arguments.this()).is_none() {
        return;
    }
    let options = options_argument(arguments);
    match super::xr_projection_layer::create(scope, options) {
        Ok(layer) => result.set(layer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn create_quad_layer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if require(scope, arguments.this()).is_none() {
        return;
    }
    let options = options_argument(arguments);
    match super::xr_quad_layer::create(scope, options) {
        Ok(layer) => result.set(layer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_sub_image(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(binding) = require(scope, arguments.this()) else {
        return;
    };
    let texture = v8::Local::new(scope, &binding.context);
    match super::xr_webgl_sub_image::create(scope, Some(texture)) {
        Ok(sub_image) => result.set(sub_image.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_view_sub_image(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(binding) = require(scope, arguments.this()) else {
        return;
    };
    let texture = v8::Local::new(scope, &binding.context);
    match super::xr_webgl_sub_image::create(scope, Some(texture)) {
        Ok(sub_image) => result.set(sub_image.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_camera_image(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(binding) = require(scope, arguments.this()) else {
        return;
    };
    result.set(v8::Local::new(scope, &binding.context));
}

fn get_depth_information(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(binding) = require(scope, arguments.this()) else {
        return;
    };
    let texture = v8::Local::new(scope, &binding.context);
    match super::xr_webgl_depth_information::create(scope, texture) {
        Ok(information) => result.set(information.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_reflection_cube_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(binding) = require(scope, arguments.this()) else {
        return;
    };
    result.set(v8::Local::new(scope, &binding.context));
}
