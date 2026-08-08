#[derive(Default)]
pub(crate) struct SvgAnimateTransformElementStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SvgAnimateTransformElementStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "SVGAnimateTransformElement", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = s
        .get_slot::<SvgAnimateTransformElementStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(s, &old));
    }
    let c = crate::webidl::create_function(
        s,
        "SVGAnimateTransformElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::svg_animation_element::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SvgAnimateTransformElementStore>()
        .ok_or_else(|| "SVGAnimateTransformElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    super::svg_animation_element::create_with_constructor(s, c, "animateTransform", owner)
}
fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'SVGAnimateTransformElement': Illegal constructor",
    )
}
