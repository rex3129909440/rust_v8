use std::collections::HashMap;

#[derive(Clone)]
struct CompressorRecord {
    threshold: v8::Global<v8::Object>,
    knee: v8::Global<v8::Object>,
    ratio: v8::Global<v8::Object>,
    attack: v8::Global<v8::Object>,
    release: v8::Global<v8::Object>,
    reduction: f64,
}

#[derive(Default)]
pub(crate) struct DynamicsCompressorNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CompressorRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DynamicsCompressorNodeStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DynamicsCompressorNode", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<DynamicsCompressorNodeStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "DynamicsCompressorNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::audio_node::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "threshold", get_threshold)?;
    crate::webidl::define_readonly_accessor(s, p, "knee", get_knee)?;
    crate::webidl::define_readonly_accessor(s, p, "ratio", get_ratio)?;
    crate::webidl::define_readonly_accessor(s, p, "reduction", get_reduction)?;
    crate::webidl::define_readonly_accessor(s, p, "attack", get_attack)?;
    crate::webidl::define_readonly_accessor(s, p, "release", get_release)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<DynamicsCompressorNodeStore>()
        .ok_or_else(|| "DynamicsCompressorNode state was not prepared".to_owned())?
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
        .ok_or_else(|| "cannot create DynamicsCompressorNode".to_owned())
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'DynamicsCompressorNode': 1 argument required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    };
    if !super::base_audio_context::is_context(s, context) {
        crate::webidl::throw_type_error(s, "parameter 1 is not of type 'BaseAudioContext'");
        return;
    }
    match attach(s, a.this(), context) {
        Ok(()) => r.set(a.this().into()),
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}
fn attach(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    context: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    super::audio_node::attach(s, o, Some(context), 1, 1);
    let threshold = super::audio_param::create(s, context, -24.0, -100.0, 0.0)?;
    let knee = super::audio_param::create(s, context, 30.0, 0.0, 40.0)?;
    let ratio = super::audio_param::create(s, context, 12.0, 1.0, 20.0)?;
    let attack = super::audio_param::create(s, context, 0.003, 0.0, 1.0)?;
    let release = super::audio_param::create(s, context, 0.25, 0.0, 1.0)?;
    let record = CompressorRecord {
        threshold: v8::Global::new(s, threshold),
        knee: v8::Global::new(s, knee),
        ratio: v8::Global::new(s, ratio),
        attack: v8::Global::new(s, attack),
        release: v8::Global::new(s, release),
        reduction: 0.0,
    };
    s.get_slot_mut::<DynamicsCompressorNodeStore>()
        .ok_or_else(|| "DynamicsCompressorNode state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<CompressorRecord> {
    s.get_slot::<DynamicsCompressorNodeStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn object_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&CompressorRecord) -> v8::Global<v8::Object>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &select(&x)).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_threshold(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_get(s, a, r, |x| x.threshold.clone())
}
fn get_knee(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_get(s, a, r, |x| x.knee.clone())
}
fn get_ratio(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_get(s, a, r, |x| x.ratio.clone())
}
fn get_attack(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_get(s, a, r, |x| x.attack.clone())
}
fn get_release(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    object_get(s, a, r, |x| x.release.clone())
}
fn get_reduction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.reduction).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) struct CompressorParameters {
    pub(crate) threshold: f32,
    pub(crate) knee: f32,
    pub(crate) ratio: f32,
    pub(crate) attack: f32,
    pub(crate) release: f32,
}

pub(crate) fn parameters_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<CompressorParameters> {
    let record = record(scope, object)?;
    Some(CompressorParameters {
        threshold: super::audio_param::value_at(
            scope,
            v8::Local::new(scope, &record.threshold),
            time,
        )?,
        knee: super::audio_param::value_at(scope, v8::Local::new(scope, &record.knee), time)?,
        ratio: super::audio_param::value_at(scope, v8::Local::new(scope, &record.ratio), time)?,
        attack: super::audio_param::value_at(scope, v8::Local::new(scope, &record.attack), time)?,
        release: super::audio_param::value_at(scope, v8::Local::new(scope, &record.release), time)?,
    })
}

pub(crate) fn set_reduction(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    reduction: f64,
) {
    if let Some(record) = scope
        .get_slot_mut::<DynamicsCompressorNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.reduction = reduction;
    }
}
