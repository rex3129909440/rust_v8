use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ContentVisibilityAutoStateChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) skipped: HashMap<i32, bool>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ContentVisibilityAutoStateChangeEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "ContentVisibilityAutoStateChangeEvent",
        constructor.into(),
    )
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ContentVisibilityAutoStateChangeEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ContentVisibilityAutoStateChangeEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::content_visibility_auto_state_change_event_skipped_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ContentVisibilityAutoStateChangeEventStore>()
        .ok_or_else(|| "ContentVisibilityAutoStateChangeEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn option_bool(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> bool {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return false;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return false;
    };
    object
        .get(scope, key.into())
        .is_some_and(|value| value.boolean_value(scope))
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "ContentVisibilityAutoStateChangeEvent requires an event type",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = arguments.get(1);
    let bubbles = option_bool(scope, options, "bubbles");
    let cancelable = option_bool(scope, options, "cancelable");
    let composed = option_bool(scope, options, "composed");
    let skipped = option_bool(scope, options, "skipped");
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<ContentVisibilityAutoStateChangeEventStore>()
        .expect("ContentVisibilityAutoStateChangeEvent state")
        .skipped
        .insert(arguments.this().get_identity_hash().get(), skipped);
    result.set(arguments.this().into());
}

pub(crate) fn get_skipped(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(skipped) = scope
        .get_slot::<ContentVisibilityAutoStateChangeEventStore>()
        .and_then(|store| {
            store
                .skipped
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Boolean::new(scope, *skipped).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
