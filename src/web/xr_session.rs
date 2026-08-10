use std::collections::HashMap;

#[derive(Clone)]
struct SessionRecord {
    object: v8::Global<v8::Object>,
    context: v8::Global<v8::Context>,
    realm_id: i32,
    mode: String,
    ended: bool,
    depth_active: bool,
    next_animation_frame: u32,
    animation_frame_due_ms: Option<f64>,
    animation_frames: HashMap<u32, v8::Global<v8::Function>>,
    render_state: v8::Global<v8::Object>,
    input_sources: v8::Global<v8::Object>,
    enabled_features: v8::Global<v8::Array>,
    handlers: HashMap<String, v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct XrSessionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SessionRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrSessionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRSession", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrSessionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRSession",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "environmentBlendMode",
        get_environment_blend_mode,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "interactionMode",
        get_interaction_mode,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "visibilityState",
        get_visibility_state,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "renderState", get_render_state)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "inputSources", get_input_sources)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "domOverlayState",
        get_dom_overlay_state,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "preferredReflectionFormat",
        get_preferred_reflection_format,
    )?;
    define_handler_accessor(scope, prototype, "onend")?;
    define_handler_accessor(scope, prototype, "onselect")?;
    define_handler_accessor(scope, prototype, "oninputsourceschange")?;
    define_handler_accessor(scope, prototype, "onselectstart")?;
    define_handler_accessor(scope, prototype, "onselectend")?;
    define_handler_accessor(scope, prototype, "onvisibilitychange")?;
    define_handler_accessor(scope, prototype, "onsqueeze")?;
    define_handler_accessor(scope, prototype, "onsqueezestart")?;
    define_handler_accessor(scope, prototype, "onsqueezeend")?;
    crate::webidl::define_readonly_accessor(scope, prototype, "depthUsage", get_depth_usage)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "depthDataFormat",
        get_depth_data_format,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "depthType", get_depth_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "depthActive", get_depth_active)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "cancelAnimationFrame",
        1,
        cancel_animation_frame,
    )?;
    crate::webidl::define_method(scope, prototype, "end", 0, end)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "pauseDepthSensing",
        0,
        pause_depth_sensing,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "requestAnimationFrame",
        1,
        request_animation_frame,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "requestHitTestSource",
        1,
        request_hit_test_source,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "requestHitTestSourceForTransientInput",
        1,
        request_transient_hit_test_source,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "requestLightProbe",
        0,
        request_light_probe,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "requestReferenceSpace",
        1,
        request_reference_space,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "resumeDepthSensing",
        0,
        resume_depth_sensing,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "updateRenderState",
        0,
        update_render_state,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "enabledFeatures",
        get_enabled_features,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxRenderLayers",
        get_max_render_layers,
    )?;
    define_handler_accessor(scope, prototype, "onvisibilitymaskchange")?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "initiateRoomCapture",
        0,
        initiate_room_capture,
    )?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrSessionStore>()
        .ok_or_else(|| "XRSession state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_handler_accessor(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let getter_data = crate::webidl::string(scope, name)?;
    let getter = crate::webidl::create_function_with_data(
        scope,
        &format!("get {name}"),
        0,
        v8::ConstructorBehavior::Throw,
        get_handler,
        getter_data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, getter, &format!("{owner}.get {name}"));
    }
    let setter_data = crate::webidl::string(scope, name)?;
    let setter = crate::webidl::create_function_with_data(
        scope,
        &format!("set {name}"),
        1,
        v8::ConstructorBehavior::Throw,
        set_handler,
        setter_data.into(),
    )?;
    if let Some(owner) = crate::trace::native_label_for_value(scope, prototype.into()) {
        crate::trace::relabel_native_function(scope, setter, &format!("{owner}.set {name}"));
    }
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, name)?;
    if prototype.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err(format!("cannot define XRSession.{name}"))
    }
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
    mode: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRSession".to_owned());
    }
    super::event_target::attach(scope, object);
    let render_state = super::xr_render_state::create(scope, None)?;
    let input_sources = super::xr_input_source_array::create(scope, Vec::new())?;
    let enabled_features = v8::Array::new(scope, 1);
    let local_floor = v8::String::new(scope, "local-floor").expect("short XR feature");
    let _ = enabled_features.set_index(scope, 0, local_floor.into());
    let record = SessionRecord {
        object: v8::Global::new(scope, object),
        context: v8::Global::new(scope, scope.get_current_context()),
        realm_id: crate::webidl::realm_id(scope),
        mode,
        ended: false,
        depth_active: true,
        next_animation_frame: 1,
        animation_frame_due_ms: None,
        animation_frames: HashMap::new(),
        render_state: v8::Global::new(scope, render_state),
        input_sources: v8::Global::new(scope, input_sources),
        enabled_features: v8::Global::new(scope, enabled_features),
        handlers: HashMap::new(),
    };
    scope
        .get_slot_mut::<XrSessionStore>()
        .ok_or_else(|| "XRSession state missing".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SessionRecord> {
    scope
        .get_slot::<XrSessionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn text(scope: &mut v8::PinScope<'_, '_>, value: &str, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into())
    }
}

fn get_environment_blend_mode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(session) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mode = if session.mode == "immersive-ar" {
        "alpha-blend"
    } else {
        "opaque"
    };
    text(scope, mode, result);
}

fn get_interaction_mode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(session) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mode = if session.mode == "inline" {
        "screen-space"
    } else {
        "world-space"
    };
    text(scope, mode, result);
}

fn get_visibility_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        text(scope, "visible", result)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_render_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(session) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &session.render_state).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_input_sources(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(session) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &session.input_sources).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_dom_overlay_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_preferred_reflection_format(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        text(scope, "srgba8", result)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(session) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(
        scope,
        crate::trace::native_callback_data(scope, &arguments),
    );
    let handler = session.handlers.get(&name).cloned();
    super::window_event_handler_support::return_handler(scope, handler, result);
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(
        scope,
        crate::trace::native_callback_data(scope, &arguments),
    );
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    let Some(session) = scope.get_slot_mut::<XrSessionStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = handler {
        session.handlers.insert(name, handler);
    } else {
        session.handlers.remove(&name);
    }
}

fn get_depth_usage(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        text(scope, "cpu-optimized", result)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_depth_data_format(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        text(scope, "luminance-alpha", result)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_depth_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        text(scope, "cpu-optimized", result)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_depth_active(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(session) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, session.depth_active).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into())
    }
}

fn resolve_undefined(scope: &mut v8::PinScope<'_, '_>, result: v8::ReturnValue<'_>) {
    let undefined = v8::undefined(scope);
    resolve(scope, undefined.into(), result);
}

fn cancel_animation_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let handle = arguments.get(0).uint32_value(scope).unwrap_or(0);
    let Some(session) = scope.get_slot_mut::<XrSessionStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    session.animation_frames.remove(&handle);
    if session.animation_frames.is_empty() {
        session.animation_frame_due_ms = None;
    }
    result.set(v8::undefined(scope).into());
}

fn end(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(session) = scope.get_slot_mut::<XrSessionStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    session.ended = true;
    session.animation_frames.clear();
    session.animation_frame_due_ms = None;
    if let Ok(event) = super::event::create(scope, "end") {
        super::event_target::dispatch(scope, arguments.this(), event);
    }
    resolve_undefined(scope, result);
}

fn pause_depth_sensing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(session) = scope.get_slot_mut::<XrSessionStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    session.depth_active = false;
    resolve_undefined(scope, result);
}

fn request_animation_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let callback = v8::Global::new(scope, callback);
    let due_ms = crate::determinism::monotonic_snapshot_milliseconds(scope) + 1_000.0 / 60.0;
    let Some(session) = scope.get_slot_mut::<XrSessionStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if session.ended {
        result.set(v8::Integer::new_from_unsigned(scope, 0).into());
        return;
    }
    let handle = session.next_animation_frame;
    session.next_animation_frame = session.next_animation_frame.saturating_add(1);
    session.animation_frames.insert(handle, callback);
    session.animation_frame_due_ms.get_or_insert(due_ms);
    result.set(v8::Integer::new_from_unsigned(scope, handle).into());
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<XrSessionStore>().and_then(|store| {
        store
            .records
            .values()
            .filter(|session| !session.ended)
            .filter_map(|session| session.animation_frame_due_ms)
            .min_by(f64::total_cmp)
    })
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let monotonic_now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let mut ready_ids = scope
        .get_slot::<XrSessionStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter(|(_, session)| {
                    !session.ended
                        && session
                            .animation_frame_due_ms
                            .is_some_and(|due| due <= monotonic_now)
                })
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ready_ids.sort_unstable();
    let mut ran = false;
    for session_id in ready_ids {
        let snapshot = scope
            .get_slot_mut::<XrSessionStore>()
            .and_then(|store| store.records.get_mut(&session_id))
            .map(|session| {
                session.animation_frame_due_ms = None;
                (
                    session.object.clone(),
                    session.context.clone(),
                    session.realm_id,
                    std::mem::take(&mut session.animation_frames),
                )
            });
        let Some((session, context, realm_id, callbacks)) = snapshot else {
            continue;
        };
        if callbacks.is_empty() {
            continue;
        }
        let timestamp = super::performance::now_for_realm_at(scope, realm_id, monotonic_now)
            .unwrap_or_else(|| {
                crate::determinism::relative_high_resolution_milliseconds(scope, monotonic_now, 0.0)
            });
        let context = v8::Local::new(scope, &context);
        let callback_scope = &mut v8::ContextScope::new(scope, context);
        let session = v8::Local::new(callback_scope, &session);
        let Ok(frame) = super::xr_frame::create(callback_scope, session) else {
            continue;
        };
        let timestamp = v8::Number::new(callback_scope, timestamp);
        let callback_arguments = [timestamp.into(), frame.into()];
        let mut callbacks = callbacks.into_iter().collect::<Vec<_>>();
        callbacks.sort_by_key(|(handle, _)| *handle);
        for (_, callback) in callbacks {
            let callback = v8::Local::new(callback_scope, &callback);
            let _ = callback.call(callback_scope, session.into(), &callback_arguments);
            callback_scope.perform_microtask_checkpoint();
        }
        ran = true;
    }
    ran
}

fn request_hit_test_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::xr_hit_test_source::create(scope) {
        Ok(source) => resolve(scope, source.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn request_transient_hit_test_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::xr_transient_input_hit_test_source::create(scope) {
        Ok(source) => resolve(scope, source.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn request_light_probe(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::xr_light_probe::create(scope) {
        Ok(probe) => resolve(scope, probe.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn request_reference_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::xr_reference_space::create(scope) {
        Ok(reference_space) => resolve(scope, reference_space.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn resume_depth_sensing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(session) = scope.get_slot_mut::<XrSessionStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    session.depth_active = true;
    resolve_undefined(scope, result);
}

fn update_render_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let state = match super::xr_render_state::create(scope, options) {
        Ok(state) => state,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let state = v8::Global::new(scope, state);
    let Some(session) = scope.get_slot_mut::<XrSessionStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    session.render_state = state;
    result.set(v8::undefined(scope).into());
}

fn get_enabled_features(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(session) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &session.enabled_features).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_max_render_layers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 1).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn initiate_room_capture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        resolve_undefined(scope, result)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
