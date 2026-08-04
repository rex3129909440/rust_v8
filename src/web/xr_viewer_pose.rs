use std::collections::HashMap;

#[derive(Clone)]
struct ViewerPoseRecord {
    views: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct XrViewerPoseStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ViewerPoseRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrViewerPoseStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "XRViewerPose", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrViewerPoseStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRViewerPose",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "views", views)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::xr_pose::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrViewerPoseStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn views(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(state) = s
        .get_slot::<XrViewerPoseStore>()
        .and_then(|store| store.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    {
        r.set(v8::Local::new(s, &state.views).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create XRViewerPose".to_owned());
    }
    let view = super::xr_view::create(s)?;
    let views = v8::Array::new(s, 1);
    let _ = views.set_index(s, 0, view.into());
    let views = v8::Global::new(s, views);
    s.get_slot_mut::<XrViewerPoseStore>()
        .ok_or_else(|| "XRViewerPose state missing".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), ViewerPoseRecord { views });
    Ok(o)
}
