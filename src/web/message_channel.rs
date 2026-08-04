use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MessageChannelStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, ChannelRecord>,
}

#[derive(Clone)]
pub(crate) struct ChannelRecord {
    pub(crate) port1: v8::Global<v8::Object>,
    pub(crate) port2: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MessageChannelStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MessageChannel", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MessageChannelStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MessageChannel",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::message_channel_port1_property::define(scope, prototype)?;
    super::message_channel_port2_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MessageChannelStore>()
        .ok_or_else(|| "MessageChannel state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MessageChannel': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    let (port1, port2) = match super::message_port::create_pair(scope) {
        Ok(pair) => pair,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let port1 = v8::Global::new(scope, port1);
    let port2 = v8::Global::new(scope, port2);
    scope
        .get_slot_mut::<MessageChannelStore>()
        .expect("MessageChannel state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ChannelRecord { port1, port2 },
        );
    result.set(arguments.this().into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<MessageChannelStore>() {
        store.constructors.remove(&realm_id);
    }
}
