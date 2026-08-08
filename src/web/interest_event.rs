use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct InterestEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, InterestEventRecord>,
}

#[derive(Clone)]
pub(crate) struct InterestEventRecord {
    pub(crate) source: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(InterestEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "InterestEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<InterestEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "InterestEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::interest_event_source_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<InterestEventStore>()
        .ok_or_else(|| "InterestEvent state was not prepared".to_owned())?
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
            "Failed to construct 'InterestEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let bubbles = init.is_some_and(|value| super::event::boolean_property(scope, value, "bubbles"));
    let cancelable =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "cancelable"));
    let composed =
        init.is_some_and(|value| super::event::boolean_property(scope, value, "composed"));
    let source = match init.and_then(|value| object_property(scope, value, "source")) {
        Some(source) if super::element::record(scope, source).is_some() => {
            Some(v8::Global::new(scope, source))
        }
        Some(_) => {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'InterestEvent': Failed to convert source to 'Element'",
            );
            return;
        }
        None => None,
    };
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<InterestEventStore>()
        .expect("InterestEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            InterestEventRecord { source },
        );
    result.set(arguments.this().into());
}

pub(crate) fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_null_or_undefined() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(value).ok()
    }
}

pub(crate) fn get_source(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let record = scope
        .get_slot::<InterestEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    match record {
        Some(record) => match record.source {
            Some(source) => result.set(v8::Local::new(scope, &source).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
