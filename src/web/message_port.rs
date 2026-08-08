use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MessagePortStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, MessagePortRecord>,
}

#[derive(Clone)]
pub(crate) struct MessagePortRecord {
    pub(crate) object: v8::Global<v8::Object>,
    pub(crate) context: v8::Global<v8::Context>,
    pub(crate) peer: Option<i32>,
    pub(crate) onmessage: Option<v8::Global<v8::Value>>,
    pub(crate) onmessageerror: Option<v8::Global<v8::Value>>,
    pub(crate) started: bool,
    pub(crate) closed: bool,
    pub(crate) detached: bool,
    pub(crate) pending: Vec<QueuedMessage>,
}

#[derive(Clone)]
pub(crate) struct QueuedMessage {
    pub(crate) data: v8::Global<v8::Value>,
    pub(crate) ports: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MessagePortStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MessagePort", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MessagePortStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "MessagePort",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::message_port_onmessage_property::define(scope, prototype)?;
    super::message_port_onmessageerror_property::define(scope, prototype)?;
    super::message_port_close::define(scope, prototype)?;
    super::message_port_post_message::define(scope, prototype)?;
    super::message_port_start::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MessagePortStore>()
        .ok_or_else(|| "MessagePort state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create_pair<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<(v8::Local<'s, v8::Object>, v8::Local<'s, v8::Object>), String> {
    let first = create(scope)?;
    let second = create(scope)?;
    let first_id = first.get_identity_hash().get();
    let second_id = second.get_identity_hash().get();
    let Some(store) = scope.get_slot_mut::<MessagePortStore>() else {
        return Err("MessagePort state was not prepared".to_owned());
    };
    if let Some(record) = store.records.get_mut(&first_id) {
        record.peer = Some(second_id);
    }
    if let Some(record) = store.records.get_mut(&second_id) {
        record.peer = Some(first_id);
    }
    Ok((first, second))
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MessagePort".to_owned());
    }
    super::event_target::attach(scope, object);
    let object_global = v8::Global::new(scope, object);
    let context = v8::Global::new(scope, scope.get_entered_or_microtask_context());
    scope
        .get_slot_mut::<MessagePortStore>()
        .ok_or_else(|| "MessagePort state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            MessagePortRecord {
                object: object_global,
                context,
                peer: None,
                onmessage: None,
                onmessageerror: None,
                started: false,
                closed: false,
                detached: false,
                pending: Vec::new(),
            },
        );
    Ok(object)
}

pub(crate) fn validate_transfer(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let record = scope
        .get_slot::<MessagePortStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .ok_or_else(|| "The object is not a MessagePort.".to_owned())?;
    if record.detached || record.closed {
        return Err("A detached or closed MessagePort cannot be transferred.".to_owned());
    }
    Ok(())
}

pub(crate) fn transfer_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    validate_transfer(scope, source)?;
    let source_id = source.get_identity_hash().get();
    let snapshot = scope
        .get_slot::<MessagePortStore>()
        .and_then(|store| store.records.get(&source_id))
        .cloned()
        .ok_or_else(|| "The MessagePort disappeared during transfer.".to_owned())?;
    let target = create(scope)?;
    let target_id = target.get_identity_hash().get();
    let target_context = v8::Global::new(scope, scope.get_entered_or_microtask_context());
    let store = scope
        .get_slot_mut::<MessagePortStore>()
        .ok_or_else(|| "MessagePort state was not prepared".to_owned())?;
    let target_record = store
        .records
        .get_mut(&target_id)
        .ok_or_else(|| "Transferred MessagePort target is unavailable.".to_owned())?;
    target_record.context = target_context;
    target_record.peer = snapshot.peer;
    target_record.pending = snapshot.pending;
    target_record.started = false;
    target_record.closed = false;
    target_record.detached = false;
    if let Some(peer_id) = snapshot.peer
        && let Some(peer) = store.records.get_mut(&peer_id)
    {
        peer.peer = Some(target_id);
    }
    if let Some(source_record) = store.records.get_mut(&source_id) {
        source_record.peer = None;
        source_record.pending.clear();
        source_record.onmessage = None;
        source_record.onmessageerror = None;
        source_record.started = false;
        source_record.closed = true;
        source_record.detached = true;
    }
    Ok(target)
}

pub(crate) fn adopt_transferred_object(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) {
    if let Ok(constructor) = ensure_constructor(scope)
        && let Ok(prototype) = crate::webidl::prototype(scope, constructor)
    {
        let _ = crate::webidl::set_platform_prototype(scope, object, prototype.into());
    }
    let context = v8::Global::new(scope, scope.get_entered_or_microtask_context());
    if let Some(record) = scope
        .get_slot_mut::<MessagePortStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.context = context;
    }
}

pub(crate) fn is_port(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<MessagePortStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'MessagePort': Illegal constructor",
    );
}

pub(crate) fn close_object(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let id = object.get_identity_hash().get();
    if let Some(record) = scope
        .get_slot_mut::<MessagePortStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        record.closed = true;
        record.pending.clear();
    }
}

pub(crate) fn schedule_delivery(scope: &mut v8::PinScope<'_, '_>, port_id: i32) {
    let data = v8::Integer::new(scope, port_id);
    let Some(function) = v8::Function::builder(deliver_microtask)
        .data(data.into())
        .length(0)
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope)
    else {
        return;
    };
    scope.enqueue_microtask(function);
}

fn deliver_microtask(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(port_id) = arguments.data().int32_value(scope) else {
        return;
    };
    let Some((context, target, handler, messages)) = scope
        .get_slot_mut::<MessagePortStore>()
        .and_then(|store| store.records.get_mut(&port_id))
        .and_then(|record| {
            if record.closed || !record.started || record.pending.is_empty() {
                return None;
            }
            Some((
                record.context.clone(),
                record.object.clone(),
                record.onmessage.clone(),
                std::mem::take(&mut record.pending),
            ))
        })
    else {
        return;
    };
    let context = v8::Local::new(scope, &context);
    let target_scope = &mut v8::ContextScope::new(scope, context);
    let target = v8::Local::new(target_scope, &target);
    for message in messages {
        let data = v8::Local::new(target_scope, &message.data);
        let ports = message
            .ports
            .iter()
            .map(|port| v8::Local::new(target_scope, port))
            .collect();
        let Ok(event) =
            super::message_event::create(target_scope, "message", data, "", None, ports)
        else {
            continue;
        };
        super::event_target::dispatch(target_scope, target, event);
        if let Some(handler) = &handler
            && let Ok(function) =
                v8::Local::<v8::Function>::try_from(v8::Local::new(target_scope, handler))
        {
            let _ = function.call(target_scope, target.into(), &[event.into()]);
        }
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<MessagePortStore>() {
        store.constructors.remove(&realm_id);
    }
}
