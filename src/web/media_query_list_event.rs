use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaQueryListEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, QueryEventRecord>,
}

#[derive(Clone)]
pub(crate) struct QueryEventRecord {
    pub(crate) media: String,
    pub(crate) matches: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaQueryListEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaQueryListEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaQueryListEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaQueryListEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::media_query_list_event_media_property::define(scope, prototype)?;
    super::media_query_list_event_matches_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaQueryListEventStore>()
        .ok_or_else(|| "MediaQueryListEvent state was not prepared".to_owned())?
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
            "Failed to construct 'MediaQueryListEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let media = init
        .and_then(|init| string_property(scope, init, "media"))
        .unwrap_or_default();
    let matches = init.is_some_and(|init| super::event::boolean_property(scope, init, "matches"));
    let bubbles = init.is_some_and(|init| super::event::boolean_property(scope, init, "bubbles"));
    let cancelable =
        init.is_some_and(|init| super::event::boolean_property(scope, init, "cancelable"));
    let composed = init.is_some_and(|init| super::event::boolean_property(scope, init, "composed"));
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<MediaQueryListEventStore>()
        .expect("MediaQueryListEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            QueryEventRecord { media, matches },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<QueryEventRecord> {
    scope
        .get_slot::<MediaQueryListEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.media) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_matches(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.matches).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    Some(crate::webidl::value_to_string(scope, value))
}
