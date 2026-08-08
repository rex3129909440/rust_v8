use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct BeforeInstallPromptEventRecord {
    pub(crate) platforms: v8::Global<v8::Array>,
    pub(crate) user_choice: v8::Global<v8::Promise>,
}

#[derive(Default)]
pub(crate) struct BeforeInstallPromptEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, BeforeInstallPromptEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BeforeInstallPromptEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BeforeInstallPromptEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<BeforeInstallPromptEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BeforeInstallPromptEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::before_install_prompt_event_platforms_property::define(scope, prototype)?;
    super::before_install_prompt_event_user_choice_property::define(scope, prototype)?;
    super::before_install_prompt_event_prompt::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BeforeInstallPromptEventStore>()
        .ok_or_else(|| "BeforeInstallPromptEvent state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "BeforeInstallPromptEvent requires a type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let platforms = v8::Array::new(scope, 0);
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let choice = v8::Object::new(scope);
    let outcome_key = v8::String::new(scope, "outcome").expect("outcome key");
    let outcome = v8::String::new(scope, "dismissed").expect("outcome");
    let _ = choice.set(scope, outcome_key.into(), outcome.into());
    let platform_key = v8::String::new(scope, "platform").expect("platform key");
    let platform = v8::String::new(scope, "").expect("platform");
    let _ = choice.set(scope, platform_key.into(), platform.into());
    let _ = resolver.resolve(scope, choice.into());
    let record = BeforeInstallPromptEventRecord {
        platforms: v8::Global::new(scope, platforms),
        user_choice: v8::Global::new(scope, resolver.get_promise(scope)),
    };
    scope
        .get_slot_mut::<BeforeInstallPromptEventStore>()
        .expect("BeforeInstallPromptEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<BeforeInstallPromptEventRecord> {
    scope
        .get_slot::<BeforeInstallPromptEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_platforms(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.platforms).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_user_choice(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.user_choice).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn prompt(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let undefined = v8::undefined(scope);
    let _ = resolver.resolve(scope, undefined.into());
    result.set(resolver.get_promise(scope).into());
}
