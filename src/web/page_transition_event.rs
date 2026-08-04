use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PageTransitionEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) persisted: HashMap<i32, bool>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PageTransitionEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PageTransitionEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PageTransitionEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PageTransitionEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::page_transition_event_persisted_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PageTransitionEventStore>()
        .ok_or_else(|| "PageTransitionEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PageTransitionEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let persisted = init
        .map(|init| super::event::boolean_property(scope, init, "persisted"))
        .unwrap_or(false);
    let bubbles = init
        .map(|init| super::event::boolean_property(scope, init, "bubbles"))
        .unwrap_or(false);
    let cancelable = init
        .map(|init| super::event::boolean_property(scope, init, "cancelable"))
        .unwrap_or(false);
    let composed = init
        .map(|init| super::event::boolean_property(scope, init, "composed"))
        .unwrap_or(false);
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<PageTransitionEventStore>()
        .expect("PageTransitionEvent state")
        .persisted
        .insert(arguments.this().get_identity_hash().get(), persisted);
    result.set(arguments.this().into());
}

pub(crate) fn get_persisted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(persisted) = scope
        .get_slot::<PageTransitionEventStore>()
        .and_then(|store| {
            store
                .persisted
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Boolean::new(scope, *persisted).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
