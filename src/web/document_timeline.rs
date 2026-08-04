use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct DocumentTimelineStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentTimelineStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DocumentTimeline", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<DocumentTimelineStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "DocumentTimeline",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::animation_timeline::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<DocumentTimelineStore>()
        .ok_or_else(|| "DocumentTimeline state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    origin: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create DocumentTimeline".to_owned());
    }
    super::animation_timeline::attach(s, o, Some(-origin), None);
    s.get_slot_mut::<DocumentTimelineStore>()
        .ok_or_else(|| "DocumentTimeline state was not prepared".to_owned())?
        .objects
        .insert(o.get_identity_hash().get());
    Ok(o)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "Please use the 'new' operator");
        return;
    }
    let origin = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .map(|options| super::event::number_property(s, options, "originTime", 0.0))
        .unwrap_or(0.0);
    super::animation_timeline::attach(s, a.this(), Some(-origin), None);
    s.get_slot_mut::<DocumentTimelineStore>()
        .expect("DocumentTimeline state")
        .objects
        .insert(a.this().get_identity_hash().get());
    r.set(a.this().into())
}
