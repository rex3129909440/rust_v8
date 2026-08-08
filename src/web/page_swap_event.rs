use std::collections::HashMap;
#[derive(Clone, Default)]
pub(crate) struct SwapData {
    pub(crate) transition: Option<v8::Global<v8::Object>>,
    pub(crate) activation: Option<v8::Global<v8::Object>>,
}
#[derive(Default)]
pub(crate) struct PageSwapEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SwapData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PageSwapEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PageSwapEvent", c.into())
}
pub(crate) fn ensure<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<PageSwapEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "PageSwapEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::page_swap_event_view_transition_property::define(s, p)?;
    super::page_swap_event_activation_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PageSwapEventStore>()
        .ok_or_else(|| "PageSwapEvent state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'PageSwapEvent': 1 argument required, but only 0 present.",
        );
        return;
    }
    let ty = crate::webidl::value_to_string(s, a.get(0));
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let data = SwapData {
        transition: obj_property(s, init, "viewTransition").map(|v| v8::Global::new(s, v)),
        activation: obj_property(s, init, "activation").map(|v| v8::Global::new(s, v)),
    };
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(s, a.this(), ty, bubbles, cancelable, composed);
    s.get_slot_mut::<PageSwapEventStore>()
        .expect("state")
        .records
        .insert(a.this().get_identity_hash().get(), data);
    r.set(a.this().into())
}
pub(crate) fn get_transition(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get(s, a, r, |x| x.transition)
}
pub(crate) fn get_activation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get(s, a, r, |x| x.activation)
}
pub(crate) fn get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(SwapData) -> Option<v8::Global<v8::Object>>,
) {
    let Some(x) = s
        .get_slot::<PageSwapEventStore>()
        .and_then(|x| x.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    match f(x) {
        Some(v) => r.set(v8::Local::new(s, &v).into()),
        None => r.set(v8::null(s).into()),
    }
}
pub(crate) fn obj_property<'s>(
    s: &v8::PinScope<'s, '_>,
    o: Option<v8::Local<'_, v8::Object>>,
    n: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let k = v8::String::new(s, n)?;
    v8::Local::<v8::Object>::try_from(o?.get(s, k.into())?).ok()
}
