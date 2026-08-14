#[derive(Default)]
pub(crate) struct LanguageModelStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LanguageModelStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    if crate::browser_surface::current_version(scope).major() < 148 {
        return Ok(());
    }
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "LanguageModel", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<LanguageModelStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let event_target = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "LanguageModel",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, event_target)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "contextUsage", illegal_getter)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "contextWindow", illegal_getter)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncontextoverflow",
        illegal_getter,
        illegal_setter,
    )?;
    crate::webidl::define_method(scope, prototype, "append", 1, illegal_append)?;
    crate::webidl::define_method(scope, prototype, "clone", 0, illegal_clone)?;
    crate::webidl::define_method(scope, prototype, "destroy", 0, illegal_plain)?;
    crate::webidl::define_method(scope, prototype, "measureContextUsage", 1, illegal_measure)?;
    crate::webidl::define_method(scope, prototype, "prompt", 1, illegal_prompt)?;
    crate::webidl::define_method(scope, prototype, "promptStreaming", 1, illegal_plain)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "availability", 0, availability)?;
    crate::webidl::define_method(scope, constructor.into(), "create", 0, create)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<LanguageModelStore>()
        .ok_or_else(|| "LanguageModel state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let message = if arguments.is_construct_call() {
        "Failed to construct 'LanguageModel': Illegal constructor"
    } else {
        "Illegal constructor"
    };
    crate::webidl::throw_type_error(scope, message)
}

fn availability(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = v8::String::new(scope, "unavailable") else {
        return;
    };
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn create(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let exception = super::dom_exception::create(
        scope,
        "Unable to create a text session because the service is not running.".to_owned(),
        "NotSupportedError".to_owned(),
    )
    .map(Into::into)
    .unwrap_or_else(|_| v8::undefined(scope).into());
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
        result.set(promise.into());
    }
}

fn illegal_getter(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal invocation")
}

fn illegal_setter(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal invocation")
}

fn illegal_plain(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal invocation")
}

fn illegal_append(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to execute 'append' on 'LanguageModel': Illegal invocation",
    )
}

fn illegal_clone(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to execute 'clone' on 'LanguageModel': Illegal invocation",
    )
}

fn illegal_measure(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to execute 'measureContextUsage' on 'LanguageModel': Illegal invocation",
    )
}

fn illegal_prompt(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to execute 'prompt' on 'LanguageModel': Illegal invocation",
    )
}
