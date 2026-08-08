use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct StereoPannerNodeStore {
    constructor: crate::webidl::RealmConstructor,
    pans: HashMap<i32, v8::Global<v8::Object>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StereoPannerNodeStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "StereoPannerNode", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StereoPannerNodeStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "StereoPannerNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "pan", get_pan)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StereoPannerNodeStore>()
        .ok_or_else(|| "StereoPannerNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    constructor
        .new_instance(scope, &[context.into()])
        .ok_or_else(|| "cannot create StereoPannerNode".to_owned())
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(scope, "StereoPannerNode requires an AudioContext");
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let pan_value = options
        .and_then(|o| v8::String::new(scope, "pan").and_then(|k| o.get(scope, k.into())))
        .filter(|v| !v.is_undefined())
        .and_then(|v| v.number_value(scope))
        .unwrap_or(0.0) as f32;
    let pan = match super::audio_param::create(scope, context, pan_value, -1.0, 1.0) {
        Ok(v) => v,
        Err(m) => {
            crate::webidl::throw_type_error(scope, &m);
            return;
        }
    };
    super::audio_node::attach(scope, a.this(), Some(context), 1, 1);
    let pan = v8::Global::new(scope, pan);
    scope
        .get_slot_mut::<StereoPannerNodeStore>()
        .expect("StereoPannerNode state")
        .pans
        .insert(a.this().get_identity_hash().get(), pan);
    r.set(a.this().into())
}
fn get_pan(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = scope
        .get_slot::<StereoPannerNodeStore>()
        .and_then(|s| s.pans.get(&a.this().get_identity_hash().get()))
        .cloned()
    {
        r.set(v8::Local::new(scope, &v).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn pan_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<f32> {
    let pan = scope
        .get_slot::<StereoPannerNodeStore>()?
        .pans
        .get(&object.get_identity_hash().get())?;
    super::audio_param::value_at(scope, v8::Local::new(scope, pan), time)
}
