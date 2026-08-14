use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct PannerNodeStore {
    constructor: crate::webidl::RealmConstructor,
    context_identities: HashSet<i32>,
    records: HashMap<i32, PannerRecord>,
}

#[derive(Clone)]
struct PannerRecord {
    panning_model: String,
    position_x: v8::Global<v8::Object>,
    position_y: v8::Global<v8::Object>,
    position_z: v8::Global<v8::Object>,
    orientation_x: v8::Global<v8::Object>,
    orientation_y: v8::Global<v8::Object>,
    orientation_z: v8::Global<v8::Object>,
    distance_model: String,
    ref_distance: f64,
    max_distance: f64,
    rolloff_factor: f64,
    cone_inner_angle: f64,
    cone_outer_angle: f64,
    cone_outer_gain: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PannerNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PannerNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PannerNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PannerNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "panningModel",
        get_panning_model,
        set_panning_model,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "positionX", get_position_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "positionY", get_position_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "positionZ", get_position_z)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "orientationX", get_orientation_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "orientationY", get_orientation_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "orientationZ", get_orientation_z)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "distanceModel",
        get_distance_model,
        set_distance_model,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "refDistance",
        get_ref_distance,
        set_ref_distance,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "maxDistance",
        get_max_distance,
        set_max_distance,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "rolloffFactor",
        get_rolloff_factor,
        set_rolloff_factor,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "coneInnerAngle",
        get_cone_inner_angle,
        set_cone_inner_angle,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "coneOuterAngle",
        get_cone_outer_angle,
        set_cone_outer_angle,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "coneOuterGain",
        get_cone_outer_gain,
        set_cone_outer_gain,
    )?;
    crate::webidl::define_method(scope, prototype, "setOrientation", 3, set_orientation)?;
    crate::webidl::define_method(scope, prototype, "setPosition", 3, set_position)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PannerNodeStore>()
        .ok_or_else(|| "PannerNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn register_context(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) {
    if let Some(store) = scope.get_slot_mut::<PannerNodeStore>() {
        store
            .context_identities
            .insert(context.get_identity_hash().get());
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PannerNode': 1 argument required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PannerNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    let valid = scope.get_slot::<PannerNodeStore>().is_some_and(|store| {
        store
            .context_identities
            .contains(&context.get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PannerNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    match attach(scope, arguments.this(), context, options) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    register_context(scope, context);
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let panner = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, panner, prototype.into()) != Some(true) {
        return Err("cannot create PannerNode".to_owned());
    }
    attach(scope, panner, context, options)?;
    Ok(panner)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
    options: Option<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let position_x = super::audio_param::create(
        scope,
        context,
        option_number(scope, options, "positionX", 0.0) as f32,
        -f32::MAX,
        f32::MAX,
    )?;
    let position_y = super::audio_param::create(
        scope,
        context,
        option_number(scope, options, "positionY", 0.0) as f32,
        -f32::MAX,
        f32::MAX,
    )?;
    let position_z = super::audio_param::create(
        scope,
        context,
        option_number(scope, options, "positionZ", 0.0) as f32,
        -f32::MAX,
        f32::MAX,
    )?;
    let orientation_x = super::audio_param::create(
        scope,
        context,
        option_number(scope, options, "orientationX", 1.0) as f32,
        -f32::MAX,
        f32::MAX,
    )?;
    let orientation_y = super::audio_param::create(
        scope,
        context,
        option_number(scope, options, "orientationY", 0.0) as f32,
        -f32::MAX,
        f32::MAX,
    )?;
    let orientation_z = super::audio_param::create(
        scope,
        context,
        option_number(scope, options, "orientationZ", 0.0) as f32,
        -f32::MAX,
        f32::MAX,
    )?;
    super::audio_node::attach(scope, object, Some(context), 1, 1);
    let _ = super::audio_node::set_channel_configuration(
        scope,
        object,
        2,
        "clamped-max".to_owned(),
        "speakers".to_owned(),
    );
    let record = PannerRecord {
        panning_model: option_string(scope, options, "panningModel", "equalpower"),
        position_x: v8::Global::new(scope, position_x),
        position_y: v8::Global::new(scope, position_y),
        position_z: v8::Global::new(scope, position_z),
        orientation_x: v8::Global::new(scope, orientation_x),
        orientation_y: v8::Global::new(scope, orientation_y),
        orientation_z: v8::Global::new(scope, orientation_z),
        distance_model: option_string(scope, options, "distanceModel", "inverse"),
        ref_distance: option_number(scope, options, "refDistance", 1.0),
        max_distance: option_number(scope, options, "maxDistance", 10000.0),
        rolloff_factor: option_number(scope, options, "rolloffFactor", 1.0),
        cone_inner_angle: option_number(scope, options, "coneInnerAngle", 360.0),
        cone_outer_angle: option_number(scope, options, "coneOuterAngle", 360.0),
        cone_outer_gain: option_number(scope, options, "coneOuterGain", 0.0),
    };
    scope
        .get_slot_mut::<PannerNodeStore>()
        .ok_or_else(|| "PannerNode state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(())
}

fn option_number(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: f64,
) -> f64 {
    options
        .map(|options| super::event::number_property(scope, options, name, default))
        .unwrap_or(default)
}

pub(crate) struct SpatialParameters {
    pub(crate) position: [f32; 3],
    pub(crate) distance_gain: f32,
}

pub(crate) fn spatial_parameters(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<SpatialParameters> {
    let record = scope
        .get_slot::<PannerNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())?;
    let position = [
        super::audio_param::value_at(scope, v8::Local::new(scope, &record.position_x), time)?,
        super::audio_param::value_at(scope, v8::Local::new(scope, &record.position_y), time)?,
        super::audio_param::value_at(scope, v8::Local::new(scope, &record.position_z), time)?,
    ];
    let context = super::audio_node::context(scope, object)?;
    let listener = super::base_audio_context::listener(scope, context)?;
    let listener_position = super::audio_listener::position_at(scope, listener, time)?;
    let dx = f64::from(position[0] - listener_position[0]);
    let dy = f64::from(position[1] - listener_position[1]);
    let dz = f64::from(position[2] - listener_position[2]);
    let distance = dx.hypot(dy).hypot(dz);
    let reference = record.ref_distance.max(f64::EPSILON);
    let distance_gain = match record.distance_model.as_str() {
        "linear" => {
            let span = (record.max_distance - reference).max(f64::EPSILON);
            (1.0 - record.rolloff_factor * (distance - reference) / span).clamp(0.0, 1.0)
        }
        "exponential" => (distance.max(reference) / reference).powf(-record.rolloff_factor),
        _ => {
            reference / (reference + record.rolloff_factor * (distance.max(reference) - reference))
        }
    };
    Some(SpatialParameters {
        position,
        distance_gain: distance_gain as f32,
    })
}
fn option_string(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: &str,
) -> String {
    let Some(options) = options else {
        return default.to_owned();
    };
    let Some(key) = v8::String::new(scope, name) else {
        return default.to_owned();
    };
    let Some(value) = options.get(scope, key.into()) else {
        return default.to_owned();
    };
    if value.is_undefined() {
        default.to_owned()
    } else {
        crate::webidl::value_to_string(scope, value)
    }
}
fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<PannerRecord> {
    scope
        .get_slot::<PannerNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PannerRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PannerRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn return_param(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PannerRecord) -> &v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_panning_model(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.panning_model);
}
fn get_distance_model(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.distance_model);
}
fn get_position_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |v| &v.position_x);
}
fn get_position_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |v| &v.position_y);
}
fn get_position_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |v| &v.position_z);
}
fn get_orientation_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |v| &v.orientation_x);
}
fn get_orientation_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |v| &v.orientation_y);
}
fn get_orientation_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_param(s, a, r, |v| &v.orientation_z);
}
fn get_ref_distance(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.ref_distance);
}
fn get_max_distance(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.max_distance);
}
fn get_rolloff_factor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.rolloff_factor);
}
fn get_cone_inner_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.cone_inner_angle);
}
fn get_cone_outer_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.cone_outer_angle);
}
fn get_cone_outer_gain(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.cone_outer_gain);
}

fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    select: fn(&mut PannerRecord) -> &mut String,
    valid: fn(&str) -> bool,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !valid(&value) {
        crate::webidl::throw_type_error(scope, "The provided enum value is invalid");
        return;
    }
    if let Some(record) = scope.get_slot_mut::<PannerNodeStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        *select(record) = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_panning_model(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    set_string(
        scope,
        a,
        |v| &mut v.panning_model,
        |v| matches!(v, "equalpower" | "HRTF"),
    );
}
fn set_distance_model(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    set_string(
        scope,
        a,
        |v| &mut v.distance_model,
        |v| matches!(v, "linear" | "inverse" | "exponential"),
    );
}

fn set_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    select: fn(&mut PannerRecord) -> &mut f64,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if let Some(record) = scope.get_slot_mut::<PannerNodeStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        *select(record) = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_ref_distance(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.ref_distance);
}
fn set_max_distance(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.max_distance);
}
fn set_rolloff_factor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.rolloff_factor);
}
fn set_cone_inner_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.cone_inner_angle);
}
fn set_cone_outer_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.cone_outer_angle);
}
fn set_cone_outer_gain(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_number(s, a, |v| &mut v.cone_outer_gain);
}

fn set_position(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_orientation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
