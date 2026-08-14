use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigationCurrentEntryChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, NavigationCurrentEntryChangeRecord>,
}
#[derive(Clone)]
pub(crate) struct NavigationCurrentEntryChangeRecord {
    pub(crate) navigation_type: Option<String>,
    pub(crate) from: v8::Global<v8::Object>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationCurrentEntryChangeEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigationCurrentEntryChangeEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigationCurrentEntryChangeEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let c = crate::webidl::create_function(
        scope,
        "NavigationCurrentEntryChangeEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::navigation_current_entry_change_event_navigation_type_property::define(scope, p)?;
    super::navigation_current_entry_change_event_from_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NavigationCurrentEntryChangeEventStore>()
        .ok_or_else(|| "NavigationCurrentEntryChangeEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NavigationCurrentEntryChangeEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, a.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NavigationCurrentEntryChangeEvent': The provided value is not of type 'NavigationCurrentEntryChangeEventInit'.",
        );
        return;
    };
    let Some(from_key) = v8::String::new(scope, "from") else {
        return;
    };
    let Some(from_value) = init.get(scope, from_key.into()) else {
        return;
    };
    let Ok(from) = v8::Local::<v8::Object>::try_from(from_value) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NavigationCurrentEntryChangeEvent': Failed to read the 'from' property from 'NavigationCurrentEntryChangeEventInit': Required member is undefined.",
        );
        return;
    };
    if !super::navigation_history_entry::is_entry(scope, from) {
        crate::webidl::throw_type_error(scope, "from must be a NavigationHistoryEntry");
        return;
    }
    let navigation_type = property(scope, init, "navigationType")
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value));
    let bubbles = super::event::boolean_property(scope, init, "bubbles");
    let cancelable = super::event::boolean_property(scope, init, "cancelable");
    let composed = super::event::boolean_property(scope, init, "composed");
    super::event::attach(scope, a.this(), event_type, bubbles, cancelable, composed);
    let from = v8::Global::new(scope, from);
    scope
        .get_slot_mut::<NavigationCurrentEntryChangeEventStore>()
        .expect("NavigationCurrentEntryChangeEvent state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            NavigationCurrentEntryChangeRecord {
                navigation_type,
                from,
            },
        );
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    navigation_type: Option<String>,
    from: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create NavigationCurrentEntryChangeEvent".to_owned());
    }
    super::event::attach(scope, event, event_type.to_owned(), false, false, false);
    let from = v8::Global::new(scope, from);
    scope
        .get_slot_mut::<NavigationCurrentEntryChangeEventStore>()
        .ok_or_else(|| "NavigationCurrentEntryChangeEvent state was not prepared".to_owned())?
        .records
        .insert(
            event.get_identity_hash().get(),
            NavigationCurrentEntryChangeRecord {
                navigation_type,
                from,
            },
        );
    Ok(event)
}
pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationCurrentEntryChangeRecord> {
    scope
        .get_slot::<NavigationCurrentEntryChangeEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_navigation_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(scope, a.this()) {
        Some(v) => match v.navigation_type {
            Some(value) => {
                if let Some(s) = v8::String::new(scope, &value) {
                    r.set(s.into())
                }
            }
            None => r.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
pub(crate) fn get_from(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.from).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
