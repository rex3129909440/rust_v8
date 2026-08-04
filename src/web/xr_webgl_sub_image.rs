use std::collections::HashMap;

#[derive(Clone)]
struct WebGlSubImageRecord {
    color_texture: Option<v8::Global<v8::Value>>,
    depth_stencil_texture: Option<v8::Global<v8::Value>>,
    motion_vector_texture: Option<v8::Global<v8::Value>>,
    image_index: u32,
    color_texture_width: u32,
    color_texture_height: u32,
    depth_stencil_texture_width: u32,
    depth_stencil_texture_height: u32,
    motion_vector_texture_width: u32,
    motion_vector_texture_height: u32,
}

#[derive(Default)]
pub(crate) struct XrWebGlSubImageStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WebGlSubImageRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrWebGlSubImageStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRWebGLSubImage", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrWebGlSubImageStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRWebGLSubImage",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "colorTexture", get_color_texture)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "depthStencilTexture",
        get_depth_stencil_texture,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "motionVectorTexture",
        get_motion_vector_texture,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "imageIndex", get_image_index)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "colorTextureWidth",
        get_color_texture_width,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "colorTextureHeight",
        get_color_texture_height,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "depthStencilTextureWidth",
        get_depth_stencil_texture_width,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "depthStencilTextureHeight",
        get_depth_stencil_texture_height,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "motionVectorTextureWidth",
        get_motion_vector_texture_width,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "motionVectorTextureHeight",
        get_motion_vector_texture_height,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::xr_sub_image::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrWebGlSubImageStore>()
        .ok_or_else(|| "XRWebGLSubImage state missing".to_owned())?
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
    texture: Option<v8::Local<'_, v8::Value>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRWebGLSubImage".to_owned());
    }
    super::xr_sub_image::attach(scope, object)?;
    let color_texture = texture.map(|value| v8::Global::new(scope, value));
    scope
        .get_slot_mut::<XrWebGlSubImageStore>()
        .ok_or_else(|| "XRWebGLSubImage state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            WebGlSubImageRecord {
                color_texture,
                depth_stencil_texture: None,
                motion_vector_texture: None,
                image_index: 0,
                color_texture_width: 1280,
                color_texture_height: 720,
                depth_stencil_texture_width: 1280,
                depth_stencil_texture_height: 720,
                motion_vector_texture_width: 1280,
                motion_vector_texture_height: 720,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<WebGlSubImageRecord> {
    scope
        .get_slot::<XrWebGlSubImageStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn optional_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: fn(&WebGlSubImageRecord) -> Option<&v8::Global<v8::Value>>,
) {
    let Some(state) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&state) {
        result.set(v8::Local::new(scope, value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn unsigned_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: fn(&WebGlSubImageRecord) -> u32,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, select(&state)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_color_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    optional_value(scope, arguments, result, |state| {
        state.color_texture.as_ref()
    })
}

fn get_depth_stencil_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    optional_value(scope, arguments, result, |state| {
        state.depth_stencil_texture.as_ref()
    })
}

fn get_motion_vector_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    optional_value(scope, arguments, result, |state| {
        state.motion_vector_texture.as_ref()
    })
}

fn get_image_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    unsigned_value(scope, arguments, result, |state| state.image_index)
}

fn get_color_texture_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    unsigned_value(scope, arguments, result, |state| state.color_texture_width)
}

fn get_color_texture_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    unsigned_value(scope, arguments, result, |state| state.color_texture_height)
}

fn get_depth_stencil_texture_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    unsigned_value(scope, arguments, result, |state| {
        state.depth_stencil_texture_width
    })
}

fn get_depth_stencil_texture_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    unsigned_value(scope, arguments, result, |state| {
        state.depth_stencil_texture_height
    })
}

fn get_motion_vector_texture_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    unsigned_value(scope, arguments, result, |state| {
        state.motion_vector_texture_width
    })
}

fn get_motion_vector_texture_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    unsigned_value(scope, arguments, result, |state| {
        state.motion_vector_texture_height
    })
}
