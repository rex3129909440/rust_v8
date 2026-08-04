use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SubmitEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) submitters: HashMap<i32, Option<v8::Global<v8::Object>>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SubmitEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SubmitEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SubmitEventStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "SubmitEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::submit_event_submitter_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SubmitEventStore>()
        .ok_or_else(|| "SubmitEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(scope, "SubmitEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, a.get(0));
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let submitter = init
        .and_then(|o| v8::String::new(scope, "submitter").and_then(|k| o.get(scope, k.into())))
        .and_then(|v| {
            if v.is_null() || v.is_undefined() {
                None
            } else {
                v8::Local::<v8::Object>::try_from(v).ok()
            }
        })
        .map(|v| v8::Global::new(scope, v));
    let bubbles = init.is_some_and(|o| super::event::boolean_property(scope, o, "bubbles"));
    let cancelable = init.is_some_and(|o| super::event::boolean_property(scope, o, "cancelable"));
    let composed = init.is_some_and(|o| super::event::boolean_property(scope, o, "composed"));
    super::event::attach(scope, a.this(), event_type, bubbles, cancelable, composed);
    scope
        .get_slot_mut::<SubmitEventStore>()
        .expect("SubmitEvent state")
        .submitters
        .insert(a.this().get_identity_hash().get(), submitter);
    r.set(a.this().into())
}
pub(crate) fn get_submitter(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = scope
        .get_slot::<SubmitEventStore>()
        .and_then(|s| s.submitters.get(&a.this().get_identity_hash().get()))
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(v) = v {
        r.set(v8::Local::new(scope, &v).into())
    } else {
        r.set(v8::null(scope).into())
    }
}
