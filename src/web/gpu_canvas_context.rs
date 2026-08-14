use std::collections::HashMap;

#[derive(Clone)]
struct CanvasContextRecord {
    canvas: v8::Global<v8::Object>,
    configuration: Option<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct GpuCanvasContextStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CanvasContextRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuCanvasContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUCanvasContext", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuCanvasContextStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUCanvasContext",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "canvas", get_canvas)?;
    crate::webidl::define_method(scope, prototype, "configure", 1, configure)?;
    crate::webidl::define_method(scope, prototype, "getConfiguration", 0, get_configuration)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getCurrentTexture",
        0,
        get_current_texture,
    )?;
    crate::webidl::define_method(scope, prototype, "unconfigure", 0, unconfigure)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuCanvasContextStore>()
        .ok_or_else(|| "GPUCanvasContext state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    canvas: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create GPUCanvasContext".to_owned());
    }
    let record = CanvasContextRecord {
        canvas: v8::Global::new(scope, canvas),
        configuration: None,
    };
    scope
        .get_slot_mut::<GpuCanvasContextStore>()
        .ok_or_else(|| "GPUCanvasContext state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CanvasContextRecord> {
    scope
        .get_slot::<GpuCanvasContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_canvas(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.canvas).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn configure(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(configuration) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "GPUCanvasContext.configure requires a descriptor");
        return;
    };
    let persistent = v8::Global::new(scope, configuration);
    if let Some(record) = scope
        .get_slot_mut::<GpuCanvasContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.configuration = Some(persistent);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_configuration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.configuration {
        Some(value) => result.set(v8::Local::new(scope, &value).into()),
        None => result.set(v8::null(scope).into()),
    }
}

fn get_current_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.configuration.is_none() {
        crate::webidl::throw_type_error(scope, "GPUCanvasContext is not configured");
        return;
    }
    match super::gpu_texture::create(scope, 1, 1, 1, 0, String::new()) {
        Ok(texture) => result.set(texture.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn unconfigure(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope
        .get_slot_mut::<GpuCanvasContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.configuration = None;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuCanvasContextStore>() {
        store.constructor.remove(realm_id);
    }
}
