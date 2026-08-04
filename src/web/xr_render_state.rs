use std::collections::HashMap;

#[derive(Clone)]
struct RenderStateRecord {
    depth_near: f64,
    depth_far: f64,
    inline_vertical_field_of_view: Option<f64>,
    base_layer: Option<v8::Global<v8::Value>>,
    layers: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct XrRenderStateStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RenderStateRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrRenderStateStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRRenderState", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrRenderStateStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRRenderState",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "depthNear", get_depth_near)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "depthFar", get_depth_far)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "inlineVerticalFieldOfView",
        get_inline_vertical_field_of_view,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "baseLayer", get_base_layer)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "layers", get_layers)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrRenderStateStore>()
        .ok_or_else(|| "XRRenderState state missing".to_owned())?
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
        return Err("cannot create XRRenderState".to_owned());
    }
    let depth_near = options
        .map(|options| super::event::number_property(scope, options, "depthNear", 0.1))
        .unwrap_or(0.1);
    let depth_far = options
        .map(|options| super::event::number_property(scope, options, "depthFar", 1000.0))
        .unwrap_or(1000.0);
    let inline_vertical_field_of_view = options.and_then(|options| {
        let key = v8::String::new(scope, "inlineVerticalFieldOfView")?;
        let value = options.get(scope, key.into())?;
        (!value.is_null() && !value.is_undefined())
            .then(|| value.number_value(scope))
            .flatten()
    });
    let base_layer = options.and_then(|options| {
        let key = v8::String::new(scope, "baseLayer")?;
        let value = options.get(scope, key.into())?;
        (!value.is_null() && !value.is_undefined()).then(|| v8::Global::new(scope, value))
    });
    let layers = options
        .and_then(|options| {
            let key = v8::String::new(scope, "layers")?;
            let value = options.get(scope, key.into())?;
            v8::Local::<v8::Array>::try_from(value).ok()
        })
        .unwrap_or_else(|| v8::Array::new(scope, 0));
    let layers = v8::Global::new(scope, layers);
    let identity = object.get_identity_hash().get();
    scope
        .get_slot_mut::<XrRenderStateStore>()
        .ok_or_else(|| "XRRenderState state missing".to_owned())?
        .records
        .insert(
            identity,
            RenderStateRecord {
                depth_near,
                depth_far,
                inline_vertical_field_of_view,
                base_layer,
                layers,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RenderStateRecord> {
    scope
        .get_slot::<XrRenderStateStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_depth_near(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, state.depth_near).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_depth_far(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, state.depth_far).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_inline_vertical_field_of_view(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(state) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(field_of_view) = state.inline_vertical_field_of_view {
        result.set(v8::Number::new(scope, field_of_view).into())
    } else {
        result.set(v8::null(scope).into())
    }
}

fn get_base_layer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(state) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(base_layer) = state.base_layer {
        result.set(v8::Local::new(scope, &base_layer))
    } else {
        result.set(v8::null(scope).into())
    }
}

fn get_layers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &state.layers).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
