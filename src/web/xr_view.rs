use std::collections::HashMap;

#[derive(Clone)]
struct ViewRecord {
    viewport_scale: f64,
    projection_matrix: v8::Global<v8::Array>,
    transform: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct XrViewStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ViewRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrViewStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRView", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrViewStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRView",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "eye", get_eye)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "recommendedViewportScale",
        get_recommended_viewport_scale,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "isFirstPersonObserver",
        get_is_first_person_observer,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "camera", get_camera)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "requestViewportScale",
        1,
        request_viewport_scale,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "index", get_index)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "projectionMatrix",
        get_projection_matrix,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transform", get_transform)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrViewStore>()
        .ok_or_else(|| "XRView state missing".to_owned())?
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
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRView".to_owned());
    }
    let projection_matrix = v8::Array::new(scope, 16);
    for index in 0..16 {
        let value = if index % 5 == 0 { 1.0 } else { 0.0 };
        let value = v8::Number::new(scope, value);
        let _ = projection_matrix.set_index(scope, index, value.into());
    }
    let transform = super::xr_rigid_transform::create(scope)?;
    let projection_matrix = v8::Global::new(scope, projection_matrix);
    let transform = v8::Global::new(scope, transform);
    let identity = object.get_identity_hash().get();
    scope
        .get_slot_mut::<XrViewStore>()
        .ok_or_else(|| "XRView state missing".to_owned())?
        .records
        .insert(
            identity,
            ViewRecord {
                viewport_scale: 1.0,
                projection_matrix,
                transform,
            },
        );
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<ViewRecord> {
    scope
        .get_slot::<XrViewStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_eye(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = v8::String::new(scope, "none").expect("short XR eye value");
    result.set(value.into());
}

fn get_recommended_viewport_scale(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(view) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, view.viewport_scale).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_is_first_person_observer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Boolean::new(scope, false).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_camera(
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

fn request_viewport_scale(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let scale = arguments
        .get(0)
        .number_value(scope)
        .unwrap_or(1.0)
        .clamp(0.1, 1.0);
    let Some(view) = scope.get_slot_mut::<XrViewStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    view.viewport_scale = scale;
    result.set(v8::undefined(scope).into());
}

fn get_index(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_projection_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(view) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &view.projection_matrix).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_transform(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(view) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &view.transform).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
