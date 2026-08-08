use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct SnapEventRecord {
    pub(crate) snap_target_block: Option<v8::Global<v8::Object>>,
    pub(crate) snap_target_inline: Option<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct SnapEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, SnapEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SnapEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SnapEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SnapEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SnapEvent",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::snap_event_snap_target_block_property::define(scope, prototype)?;
    super::snap_event_snap_target_inline_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SnapEventStore>()
        .ok_or_else(|| "SnapEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SnapEvent': Illegal constructor",
    )
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    block: Option<v8::Local<'_, v8::Object>>,
    inline: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = super::event::create(scope, event_type)?;
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create SnapEvent".to_owned());
    }
    let record = SnapEventRecord {
        snap_target_block: block.map(|value| v8::Global::new(scope, value)),
        snap_target_inline: inline.map(|value| v8::Global::new(scope, value)),
    };
    scope
        .get_slot_mut::<SnapEventStore>()
        .ok_or_else(|| "SnapEvent state is unavailable".to_owned())?
        .records
        .insert(event.get_identity_hash().get(), record);
    Ok(event)
}

pub(crate) fn return_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&SnapEventRecord) -> Option<v8::Global<v8::Object>>,
) {
    let Some(record) = scope
        .get_slot::<SnapEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match select(&record) {
        Some(value) => result.set(v8::Local::new(scope, &value).into()),
        None => result.set(v8::null(scope).into()),
    }
}

pub(crate) fn get_snap_target_block(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_target(s, a, r, |record| record.snap_target_block.clone())
}

pub(crate) fn get_snap_target_inline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_target(s, a, r, |record| record.snap_target_inline.clone())
}
