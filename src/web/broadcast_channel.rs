use std::collections::{HashMap, VecDeque};

#[derive(Clone)]
pub(crate) struct BroadcastChannelRecord {
    pub(crate) object: v8::Global<v8::Object>,
    pub(crate) context: v8::Global<v8::Context>,
    pub(crate) origin: String,
    pub(crate) name: String,
    pub(crate) onmessage: Option<v8::Global<v8::Function>>,
    pub(crate) onmessageerror: Option<v8::Global<v8::Function>>,
    pub(crate) closed: bool,
}

#[derive(Clone)]
pub(crate) struct PendingBroadcastMessage {
    pub(crate) recipient_id: i32,
    pub(crate) data: v8::Global<v8::Value>,
    pub(crate) origin: String,
}

#[derive(Default)]
pub(crate) struct BroadcastChannelStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, BroadcastChannelRecord>,
    pub(crate) pending: VecDeque<PendingBroadcastMessage>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BroadcastChannelStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BroadcastChannel", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<BroadcastChannelStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BroadcastChannel",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::broadcast_channel_name_property::define(scope, prototype)?;
    super::broadcast_channel_onmessage_property::define(scope, prototype)?;
    super::broadcast_channel_onmessageerror_property::define(scope, prototype)?;
    super::broadcast_channel_close::define(scope, prototype)?;
    super::broadcast_channel_post_message::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BroadcastChannelStore>()
        .ok_or_else(|| "BroadcastChannel state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "BroadcastChannel requires a name");
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    super::event_target::attach(scope, arguments.this());
    let object = v8::Global::new(scope, arguments.this());
    let context = scope.get_entered_or_microtask_context();
    let context_global = v8::Global::new(scope, context);
    let window = context.global(scope);
    let origin = super::html_i_frame_element::origin_for_window(scope, window);
    scope
        .get_slot_mut::<BroadcastChannelStore>()
        .expect("BroadcastChannel state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            BroadcastChannelRecord {
                object,
                context: context_global,
                origin,
                name,
                onmessage: None,
                onmessageerror: None,
                closed: false,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<BroadcastChannelStore>() {
        store.constructors.remove(&realm_id);
    }
}
