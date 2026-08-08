use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ExtendableEventStore {
    constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Vec<v8::Global<v8::Value>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ExtendableEventStore::default());
}

pub(crate) fn install_in_service_worker_realm(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ExtendableEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ExtendableEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ExtendableEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, event)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::extendable_event_wait_until::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ExtendableEventStore>()
        .ok_or_else(|| "ExtendableEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ExtendableEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "bubbles"));
    let cancelable =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "cancelable"));
    let composed =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "composed"));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<ExtendableEventStore>()
        .expect("ExtendableEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), Vec::new());
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create ExtendableEvent".to_owned());
    }
    super::event::attach(scope, event, event_type.to_owned(), false, false, false);
    scope
        .get_slot_mut::<ExtendableEventStore>()
        .ok_or_else(|| "ExtendableEvent state was not prepared".to_owned())?
        .records
        .insert(event.get_identity_hash().get(), Vec::new());
    Ok(event)
}
