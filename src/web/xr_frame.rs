use std::collections::HashMap;

#[derive(Clone)]
struct FrameRecord {
    session: v8::Global<v8::Object>,
    tracked_anchors: v8::Global<v8::Object>,
    detected_planes: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct XrFrameStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, FrameRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrFrameStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRFrame", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrFrameStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRFrame",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "session", get_session)?;
    crate::webidl::define_method(scope, prototype, "getPose", 2, get_pose)?;
    crate::webidl::define_method(scope, prototype, "getViewerPose", 1, get_viewer_pose)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "trackedAnchors",
        get_tracked_anchors,
    )?;
    crate::webidl::define_method(scope, prototype, "createAnchor", 2, create_anchor)?;
    crate::webidl::define_method(scope, prototype, "fillJointRadii", 2, fill_joint_radii)?;
    crate::webidl::define_method(scope, prototype, "fillPoses", 3, fill_poses)?;
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
        "getHitTestResults",
        1,
        get_hit_test_results,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getHitTestResultsForTransientInput",
        1,
        get_transient_hit_test_results,
    )?;
    crate::webidl::define_method(scope, prototype, "getJointPose", 2, get_joint_pose)?;
    crate::webidl::define_method(scope, prototype, "getLightEstimate", 1, get_light_estimate)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "detectedPlanes",
        get_detected_planes,
    )?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrFrameStore>()
        .ok_or_else(|| "XRFrame state missing".to_owned())?
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
    session: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRFrame".to_owned());
    }
    let tracked_anchors = super::xr_anchor_set::create(scope, Vec::new())?;
    let detected_planes = super::xr_plane_set::create(scope, Vec::new())?;
    let frame = FrameRecord {
        session: v8::Global::new(scope, session),
        tracked_anchors: v8::Global::new(scope, tracked_anchors),
        detected_planes: v8::Global::new(scope, detected_planes),
    };
    scope
        .get_slot_mut::<XrFrameStore>()
        .ok_or_else(|| "XRFrame state missing".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), frame);
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<FrameRecord> {
    scope
        .get_slot::<XrFrameStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_session(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(frame) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &frame.session).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_pose(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::xr_pose::create(scope) {
        Ok(pose) => result.set(pose.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_viewer_pose(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::xr_viewer_pose::create(scope) {
        Ok(pose) => result.set(pose.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_tracked_anchors(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(frame) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &frame.tracked_anchors).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn create_anchor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(scope, "XRFrame", "createAnchor", result);
        return;
    }
    match super::xr_anchor::create(scope) {
        Ok(anchor) => {
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, anchor.into()) {
                result.set(promise.into())
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn fill_joint_radii(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(joints) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let Ok(radii) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let length_key = v8::String::new(scope, "length").expect("short key");
    let length = joints
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    for index in 0..length {
        let radius = v8::Number::new(scope, 0.01);
        let _ = radii.set_index(scope, index, radius.into());
    }
    result.set(v8::Boolean::new(scope, true).into());
}

fn fill_poses(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(spaces) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let Ok(transforms) = v8::Local::<v8::Object>::try_from(arguments.get(2)) else {
        result.set(v8::Boolean::new(scope, false).into());
        return;
    };
    let length_key = v8::String::new(scope, "length").expect("short key");
    let length = spaces
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    for index in 0..length.saturating_mul(16) {
        let column = index % 16;
        let value = if column % 5 == 0 { 1.0 } else { 0.0 };
        let value = v8::Number::new(scope, value);
        let _ = transforms.set_index(scope, index, value.into());
    }
    result.set(v8::Boolean::new(scope, true).into());
}

fn get_depth_information(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else {
        match super::xr_cpu_depth_information::create(scope) {
            Ok(information) => result.set(information.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
    }
}

fn get_hit_test_results(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Array::new(scope, 0).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_transient_hit_test_results(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Array::new(scope, 0).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_joint_pose(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match super::xr_joint_pose::create(scope) {
        Ok(pose) => result.set(pose.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn get_light_estimate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    } else {
        match super::xr_light_estimate::create(scope) {
            Ok(estimate) => result.set(estimate.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
    }
}

fn get_detected_planes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(frame) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &frame.detected_planes).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
