use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CommandEventRecord {
    pub(crate) source: Option<v8::Global<v8::Object>>,
    pub(crate) command: String,
}

#[derive(Default)]
pub(crate) struct CommandEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, CommandEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CommandEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CommandEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CommandEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CommandEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::command_event_source_property::define(scope, prototype)?;
    super::command_event_command_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CommandEventStore>()
        .ok_or_else(|| "CommandEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn option<'s>(
    scope: &v8::PinScope<'s, '_>,
    options: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = v8::Local::<v8::Object>::try_from(options).ok()?;
    object.get(scope, v8::String::new(scope, name)?.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CommandEvent requires an event type");
        return;
    }
    let options = arguments.get(1);
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = option(scope, options, "bubbles").is_some_and(|value| value.boolean_value(scope));
    let cancelable =
        option(scope, options, "cancelable").is_some_and(|value| value.boolean_value(scope));
    let composed =
        option(scope, options, "composed").is_some_and(|value| value.boolean_value(scope));
    let source = option(scope, options, "source")
        .filter(|value| !value.is_null_or_undefined())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .map(|value| v8::Global::new(scope, value));
    let command = option(scope, options, "command")
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<CommandEventStore>()
        .expect("CommandEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CommandEventRecord { source, command },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CommandEventRecord> {
    scope
        .get_slot::<CommandEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.source {
            Some(source) => result.set(v8::Local::new(scope, &source).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn get_command(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.command) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
