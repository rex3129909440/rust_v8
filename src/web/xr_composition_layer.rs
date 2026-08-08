use std::collections::HashMap;

#[derive(Clone)]
struct CompositionLayerRecord {
    layout: String,
    blend_texture_source_alpha: bool,
    force_mono_presentation: bool,
    opacity: f64,
    mip_levels: u32,
    needs_redraw: bool,
    destroyed: bool,
}

#[derive(Default)]
pub(crate) struct XrCompositionLayerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CompositionLayerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrCompositionLayerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRCompositionLayer", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrCompositionLayerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRCompositionLayer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "layout", get_layout)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "blendTextureSourceAlpha",
        get_blend_texture_source_alpha,
        set_blend_texture_source_alpha,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "forceMonoPresentation",
        get_force_mono_presentation,
        set_force_mono_presentation,
    )?;
    crate::webidl::define_accessor(scope, prototype, "opacity", get_opacity, set_opacity)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mipLevels", get_mip_levels)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "needsRedraw", get_needs_redraw)?;
    crate::webidl::define_method(scope, prototype, "destroy", 0, destroy)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::xr_layer::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrCompositionLayerStore>()
        .ok_or_else(|| "XRCompositionLayer state missing".to_owned())?
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

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    default: &str,
) -> String {
    let Some(key) = v8::String::new(scope, name) else {
        return default.to_owned();
    };
    let Some(value) = object.get(scope, key.into()) else {
        return default.to_owned();
    };
    if value.is_undefined() {
        default.to_owned()
    } else {
        value
            .to_string(scope)
            .map(|value| value.to_rust_string_lossy(scope))
            .unwrap_or_else(|| default.to_owned())
    }
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) {
    super::xr_layer::attach(scope, object);
    let layout = options
        .map(|options| string_property(scope, options, "layout", "mono"))
        .unwrap_or_else(|| "mono".to_owned());
    let blend_texture_source_alpha = options.is_some_and(|options| {
        super::event::boolean_property(scope, options, "blendTextureSourceAlpha")
    });
    let force_mono_presentation = options.is_some_and(|options| {
        super::event::boolean_property(scope, options, "forceMonoPresentation")
    });
    let opacity = options
        .map(|options| super::event::number_property(scope, options, "opacity", 1.0))
        .unwrap_or(1.0);
    let mip_levels = options
        .map(|options| super::event::number_property(scope, options, "mipLevels", 1.0))
        .unwrap_or(1.0)
        .max(1.0) as u32;
    scope
        .get_slot_mut::<XrCompositionLayerStore>()
        .expect("XRCompositionLayer state")
        .records
        .insert(
            object.get_identity_hash().get(),
            CompositionLayerRecord {
                layout,
                blend_texture_source_alpha,
                force_mono_presentation,
                opacity,
                mip_levels,
                needs_redraw: true,
                destroyed: false,
            },
        );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CompositionLayerRecord> {
    scope
        .get_slot::<XrCompositionLayerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn require(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CompositionLayerRecord> {
    let state = record(scope, object);
    if state.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    state
}

fn get_layout(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = require(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &state.layout) {
            result.set(value.into());
        }
    }
}

fn get_blend_texture_source_alpha(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = require(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, state.blend_texture_source_alpha).into());
    }
}

fn set_blend_texture_source_alpha(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments.get(0).boolean_value(scope);
    if let Some(state) = scope
        .get_slot_mut::<XrCompositionLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.blend_texture_source_alpha = value;
        state.needs_redraw = true;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_force_mono_presentation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = require(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, state.force_mono_presentation).into());
    }
}

fn set_force_mono_presentation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments.get(0).boolean_value(scope);
    if let Some(state) = scope
        .get_slot_mut::<XrCompositionLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.force_mono_presentation = value;
        state.needs_redraw = true;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_opacity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = require(scope, arguments.this()) {
        result.set(v8::Number::new(scope, state.opacity).into());
    }
}

fn set_opacity(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if let Some(state) = scope
        .get_slot_mut::<XrCompositionLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.opacity = value.clamp(0.0, 1.0);
        state.needs_redraw = true;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_mip_levels(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = require(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, state.mip_levels).into());
    }
}

fn get_needs_redraw(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = require(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, state.needs_redraw && !state.destroyed).into());
    }
}

fn destroy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    if let Some(state) = scope
        .get_slot_mut::<XrCompositionLayerStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        state.destroyed = true;
        state.needs_redraw = false;
        result.set(v8::undefined(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
