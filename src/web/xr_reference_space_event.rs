use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XrReferenceSpaceEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(XrReferenceSpaceEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "XRReferenceSpaceEvent", c.into())
}
pub(crate) fn ensure<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<XrReferenceSpaceEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "XRReferenceSpaceEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::xr_reference_space_event_reference_space_property::define(s, p)?;
    super::xr_reference_space_event_transform_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<XrReferenceSpaceEventStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'XRReferenceSpaceEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(s, a.get(0)) else {
        return;
    };
    if !a.get(1).is_object() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'XRReferenceSpaceEvent': The provided value is not of type 'XRReferenceSpaceEventInit'.",
        );
        return;
    }
    let reference_space_missing = v8::Local::<v8::Object>::try_from(a.get(1))
        .ok()
        .and_then(|init| {
            v8::String::new(s, "referenceSpace").and_then(|key| init.get(s, key.into()))
        })
        .is_none_or(|value| value.is_undefined());
    if reference_space_missing {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'XRReferenceSpaceEvent': Failed to read the 'referenceSpace' property from 'XRReferenceSpaceEventInit': Required member is undefined.",
        );
        return;
    }
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(s, a.this(), event_type, bubbles, cancelable, composed);
    s.get_slot_mut::<XrReferenceSpaceEventStore>()
        .expect("XRReferenceSpaceEvent state")
        .instances
        .insert(a.this().get_identity_hash().get());
    r.set(a.this().into())
}
pub(crate) fn null(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    r.set(v8::null(s).into())
}
pub(crate) fn transform(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !require(s, &a) {
        return;
    }
    if let Ok(v) = super::xr_rigid_transform::create(s) {
        r.set(v.into())
    }
}
pub(crate) fn require(s: &mut v8::PinScope<'_, '_>, a: &v8::FunctionCallbackArguments<'_>) -> bool {
    let valid = s
        .get_slot::<XrReferenceSpaceEventStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains(&a.this().get_identity_hash().get())
        });
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
    valid
}
