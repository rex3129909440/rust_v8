use std::collections::HashMap;

#[derive(Clone)]
struct WebGlDepthInformationRecord {
    texture: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct XrWebGlDepthInformationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WebGlDepthInformationRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrWebGlDepthInformationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRWebGLDepthInformation", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrWebGlDepthInformationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRWebGLDepthInformation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "texture", get_texture)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::xr_depth_information::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrWebGlDepthInformationStore>()
        .ok_or_else(|| "XRWebGLDepthInformation state missing".to_owned())?
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
    texture: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRWebGLDepthInformation".to_owned());
    }
    let texture = v8::Global::new(scope, texture);
    scope
        .get_slot_mut::<XrWebGlDepthInformationStore>()
        .ok_or_else(|| "XRWebGLDepthInformation state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            WebGlDepthInformationRecord { texture },
        );
    Ok(object)
}

fn get_texture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = scope
        .get_slot::<XrWebGlDepthInformationStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    {
        result.set(v8::Local::new(scope, &state.texture));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
