use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationRecord>,
}

#[derive(Clone)]
struct NavigationRecord {
    entries: Vec<v8::Global<v8::Object>>,
    current: usize,
    transition: Option<v8::Global<v8::Object>>,
    activation: Option<v8::Global<v8::Object>>,
    onnavigate: Option<v8::Global<v8::Value>>,
    onnavigatesuccess: Option<v8::Global<v8::Value>>,
    onnavigateerror: Option<v8::Global<v8::Value>>,
    oncurrententrychange: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Navigation", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Navigation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "currentEntry", get_current_entry)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transition", get_transition)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "activation", get_activation)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "canGoBack", get_can_go_back)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "canGoForward", get_can_go_forward)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onnavigate",
        get_onnavigate,
        set_onnavigate,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onnavigatesuccess",
        get_onnavigatesuccess,
        set_onnavigatesuccess,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onnavigateerror",
        get_onnavigateerror,
        set_onnavigateerror,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncurrententrychange",
        get_oncurrententrychange,
        set_oncurrententrychange,
    )?;
    crate::webidl::define_method(scope, prototype, "back", 0, back)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forward", 0, forward)?;
    crate::webidl::define_method(scope, prototype, "navigate", 1, navigate)?;
    crate::webidl::define_method(scope, prototype, "reload", 0, reload)?;
    crate::webidl::define_method(scope, prototype, "traverseTo", 1, traverse_to)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "updateCurrentEntry",
        1,
        update_current_entry,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NavigationStore>()
        .ok_or_else(|| "Navigation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial_url: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let navigation = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, navigation, prototype.into()) != Some(true) {
        return Err("cannot create Navigation".to_owned());
    }
    super::event_target::attach(scope, navigation);
    let entry = super::navigation_history_entry::create(
        scope,
        initial_url.to_owned(),
        0,
        true,
        v8::undefined(scope).into(),
    )?;
    let activation = super::navigation_activation::create(scope, entry, None, None)?;
    let entry = v8::Global::new(scope, entry);
    let activation = v8::Global::new(scope, activation);
    scope
        .get_slot_mut::<NavigationStore>()
        .ok_or_else(|| "Navigation state was not prepared".to_owned())?
        .records
        .insert(
            navigation.get_identity_hash().get(),
            NavigationRecord {
                entries: vec![entry],
                current: 0,
                transition: None,
                activation: Some(activation),
                onnavigate: None,
                onnavigatesuccess: None,
                onnavigateerror: None,
                oncurrententrychange: None,
            },
        );
    Ok(navigation)
}

pub(crate) fn create_empty<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let navigation = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, navigation, prototype.into()) != Some(true) {
        return Err("cannot create Navigation".to_owned());
    }
    super::event_target::attach(scope, navigation);
    scope
        .get_slot_mut::<NavigationStore>()
        .ok_or_else(|| "Navigation state was not prepared".to_owned())?
        .records
        .insert(
            navigation.get_identity_hash().get(),
            NavigationRecord {
                entries: Vec::new(),
                current: 0,
                transition: None,
                activation: None,
                onnavigate: None,
                onnavigatesuccess: None,
                onnavigateerror: None,
                oncurrententrychange: None,
            },
        );
    Ok(navigation)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Navigation': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationRecord> {
    scope
        .get_slot::<NavigationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn current_entry<'s>(
    scope: &v8::PinScope<'s, '_>,
    navigation: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = record(scope, navigation)?;
    record
        .entries
        .get(record.current)
        .map(|entry| v8::Local::new(scope, entry))
}

fn get_current_entry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(entry) = current_entry(scope, arguments.this()) {
        result.set(entry.into());
    } else if record(scope, arguments.this()).is_some() {
        result.set(v8::null(scope).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_optional_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigationRecord) -> Option<&v8::Global<v8::Object>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_transition(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_optional_object(scope, arguments, result, |record| {
        record.transition.as_ref()
    });
}

fn get_activation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_optional_object(scope, arguments, result, |record| {
        record.activation.as_ref()
    });
}

fn get_can_go_back(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, record.current > 0).into());
}

fn get_can_go_forward(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, record.current + 1 < record.entries.len()).into());
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigationRecord) -> Option<&v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = select(&record) {
        result.set(v8::Local::new(scope, handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_onnavigate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_handler(scope, arguments, result, |record| {
        record.onnavigate.as_ref()
    });
}

fn get_onnavigatesuccess(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_handler(scope, arguments, result, |record| {
        record.onnavigatesuccess.as_ref()
    });
}

fn get_onnavigateerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_handler(scope, arguments, result, |record| {
        record.onnavigateerror.as_ref()
    });
}

fn get_oncurrententrychange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_handler(scope, arguments, result, |record| {
        record.oncurrententrychange.as_ref()
    });
}

fn normalized_handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    v8::Local::<v8::Function>::try_from(value)
        .ok()
        .map(|function| {
            let value: v8::Local<v8::Value> = function.into();
            v8::Global::new(scope, value)
        })
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    navigation: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    update: impl FnOnce(&mut NavigationRecord, Option<v8::Global<v8::Value>>),
) {
    let handler = normalized_handler(scope, value);
    let Some(record) = scope
        .get_slot_mut::<NavigationStore>()
        .and_then(|store| store.records.get_mut(&navigation.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    update(record, handler);
}

fn set_onnavigate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(
        scope,
        arguments.this(),
        arguments.get(0),
        |record, value| record.onnavigate = value,
    );
}

fn set_onnavigatesuccess(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(
        scope,
        arguments.this(),
        arguments.get(0),
        |record, value| record.onnavigatesuccess = value,
    );
}

fn set_onnavigateerror(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(
        scope,
        arguments.this(),
        arguments.get(0),
        |record, value| record.onnavigateerror = value,
    );
}

fn set_oncurrententrychange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(
        scope,
        arguments.this(),
        arguments.get(0),
        |record, value| record.oncurrententrychange = value,
    );
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let entries = v8::Array::new(scope, record.entries.len() as i32);
    for (index, entry) in record.entries.iter().enumerate() {
        let _ = entries.set_index(scope, index as u32, v8::Local::new(scope, entry).into());
    }
    result.set(entries.into());
}

fn navigate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "Failed to execute 'navigate': 1 argument required");
        return;
    }
    let Some(from) = current_entry(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let base = super::navigation_history_entry::url(scope, from)
        .unwrap_or_else(|| "about:blank".to_owned());
    let target_url = resolve_url(&base, &input);
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let history = options
        .and_then(|object| string_property(scope, object, "history"))
        .unwrap_or_else(|| "auto".to_owned());
    let navigation_type = if history == "replace" {
        "replace"
    } else {
        "push"
    };
    let state = options
        .and_then(|object| value_property(scope, object, "state"))
        .unwrap_or_else(|| v8::undefined(scope).into());
    let info = options
        .and_then(|object| value_property(scope, object, "info"))
        .unwrap_or_else(|| v8::undefined(scope).into());
    let from_url = super::navigation_history_entry::url(scope, from).unwrap_or_default();
    let hash_change = same_without_fragment(&from_url, &target_url) && from_url != target_url;
    let destination_index = if navigation_type == "replace" {
        super::navigation_history_entry::index(scope, from).unwrap_or(0)
    } else {
        -1
    };
    let destination = match super::navigation_destination::create(
        scope,
        String::new(),
        String::new(),
        target_url.clone(),
        destination_index,
        true,
        state,
    ) {
        Ok(destination) => destination,
        Err(error) => {
            crate::webidl::throw_type_error(scope, &error);
            return;
        }
    };
    let Ok(event) =
        super::navigate_event::create(scope, navigation_type, destination, info, hash_change)
    else {
        return;
    };
    fire_event_with_handler(scope, arguments.this(), event, |record| {
        record.onnavigate.clone()
    });
    for handler in super::navigate_event::take_handlers(scope, event) {
        let function = v8::Local::new(scope, &handler);
        let _ = function.call(scope, v8::undefined(scope).into(), &[]);
    }
    let entry = if navigation_type == "replace" {
        match super::navigation_history_entry::create(
            scope,
            target_url,
            destination_index,
            true,
            state,
        ) {
            Ok(entry) => entry,
            Err(error) => {
                crate::webidl::throw_type_error(scope, &error);
                return;
            }
        }
    } else {
        let index = record(scope, arguments.this())
            .map(|record| record.current as i32 + 1)
            .unwrap_or(0);
        match super::navigation_history_entry::create(scope, target_url, index, true, state) {
            Ok(entry) => entry,
            Err(error) => {
                crate::webidl::throw_type_error(scope, &error);
                return;
            }
        }
    };
    commit_entry(
        scope,
        arguments.this(),
        from,
        entry,
        navigation_type,
        navigation_type == "replace",
    );
    if let Ok(navigation_result) = resolved_navigation_result(scope, entry) {
        result.set(navigation_result.into());
    }
}

fn back(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    traverse_delta(scope, arguments.this(), -1, &mut result);
}

fn forward(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    traverse_delta(scope, arguments.this(), 1, &mut result);
}

fn traverse_delta(
    scope: &mut v8::PinScope<'_, '_>,
    navigation: v8::Local<'_, v8::Object>,
    delta: i32,
    result: &mut v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, navigation) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let next = snapshot.current as i32 + delta;
    if next < 0 || next >= snapshot.entries.len() as i32 {
        if let Ok(value) = rejected_navigation_result(scope, "InvalidStateError") {
            result.set(value.into());
        }
        return;
    }
    let from = v8::Local::new(scope, &snapshot.entries[snapshot.current]);
    let entry = v8::Local::new(scope, &snapshot.entries[next as usize]);
    commit_existing(scope, navigation, from, entry, next as usize, "traverse");
    if let Ok(value) = resolved_navigation_result(scope, entry) {
        result.set(value.into());
    }
}

fn traverse_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'traverseTo': 1 argument required",
        );
        return;
    }
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let target = snapshot.entries.iter().position(|entry| {
        let entry = v8::Local::new(scope, entry);
        super::navigation_history_entry::key(scope, entry).is_some_and(|value| value == key)
    });
    let Some(target) = target else {
        if let Ok(value) = rejected_navigation_result(scope, "InvalidStateError") {
            result.set(value.into());
        }
        return;
    };
    let from = v8::Local::new(scope, &snapshot.entries[snapshot.current]);
    let entry = v8::Local::new(scope, &snapshot.entries[target]);
    if target != snapshot.current {
        commit_existing(scope, arguments.this(), from, entry, target, "traverse");
    }
    if let Ok(value) = resolved_navigation_result(scope, entry) {
        result.set(value.into());
    }
}

fn reload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(entry) = current_entry(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    if let Some(state) = options.and_then(|object| value_property(scope, object, "state")) {
        super::navigation_history_entry::replace_state(scope, entry, state);
    }
    let state = super::navigation_history_entry::state(scope, entry)
        .unwrap_or_else(|| v8::undefined(scope).into());
    let url = super::navigation_history_entry::url(scope, entry).unwrap_or_default();
    let index = super::navigation_history_entry::index(scope, entry).unwrap_or(0);
    let Ok(destination) = super::navigation_destination::create(
        scope,
        super::navigation_history_entry::key(scope, entry).unwrap_or_default(),
        super::navigation_history_entry::id(scope, entry).unwrap_or_default(),
        url,
        index,
        true,
        state,
    ) else {
        return;
    };
    let Ok(event) = super::navigate_event::create(
        scope,
        "reload",
        destination,
        v8::undefined(scope).into(),
        false,
    ) else {
        return;
    };
    fire_event_with_handler(scope, arguments.this(), event, |record| {
        record.onnavigate.clone()
    });
    fire_success(scope, arguments.this(), entry, entry, Some("reload"));
    if let Ok(value) = resolved_navigation_result(scope, entry) {
        result.set(value.into());
    }
}

fn update_current_entry(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'updateCurrentEntry': 1 argument required",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "updateCurrentEntry requires an options object");
        return;
    };
    let state =
        value_property(scope, options, "state").unwrap_or_else(|| v8::undefined(scope).into());
    let Some(entry) = current_entry(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::navigation_history_entry::replace_state(scope, entry, state);
    let Ok(event) = super::navigation_current_entry_change_event::create(
        scope,
        "currententrychange",
        None,
        entry,
    ) else {
        return;
    };
    fire_event_with_handler(scope, arguments.this(), event, |record| {
        record.oncurrententrychange.clone()
    });
}

fn commit_entry(
    scope: &mut v8::PinScope<'_, '_>,
    navigation: v8::Local<'_, v8::Object>,
    from: v8::Local<'_, v8::Object>,
    entry: v8::Local<'_, v8::Object>,
    navigation_type: &str,
    replace: bool,
) {
    let id = navigation.get_identity_hash().get();
    let index = if replace {
        record(scope, navigation)
            .map(|record| record.current)
            .unwrap_or(0)
    } else {
        record(scope, navigation)
            .map(|record| record.current + 1)
            .unwrap_or(0)
    };
    if let Ok(transition) = super::navigation_transition::create(
        scope,
        navigation_type.to_owned(),
        from,
        entry,
        entry.into(),
    ) {
        let transition = v8::Global::new(scope, transition);
        if let Some(record) = scope
            .get_slot_mut::<NavigationStore>()
            .and_then(|store| store.records.get_mut(&id))
        {
            record.transition = Some(transition);
        }
    }
    let entry_global = v8::Global::new(scope, entry);
    if let Some(record) = scope
        .get_slot_mut::<NavigationStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        if replace {
            record.entries[index] = entry_global;
        } else {
            record.entries.truncate(index);
            record.entries.push(entry_global);
        }
        record.current = index;
    }
    fire_success(scope, navigation, from, entry, Some(navigation_type));
}

fn commit_existing(
    scope: &mut v8::PinScope<'_, '_>,
    navigation: v8::Local<'_, v8::Object>,
    from: v8::Local<'_, v8::Object>,
    entry: v8::Local<'_, v8::Object>,
    target: usize,
    navigation_type: &str,
) {
    let url = super::navigation_history_entry::url(scope, entry).unwrap_or_default();
    let state = super::navigation_history_entry::state(scope, entry)
        .unwrap_or_else(|| v8::undefined(scope).into());
    let Ok(destination) = super::navigation_destination::create(
        scope,
        super::navigation_history_entry::key(scope, entry).unwrap_or_default(),
        super::navigation_history_entry::id(scope, entry).unwrap_or_default(),
        url,
        target as i32,
        true,
        state,
    ) else {
        return;
    };
    let Ok(event) = super::navigate_event::create(
        scope,
        navigation_type,
        destination,
        v8::undefined(scope).into(),
        false,
    ) else {
        return;
    };
    fire_event_with_handler(scope, navigation, event, |record| record.onnavigate.clone());
    if let Some(record) = scope
        .get_slot_mut::<NavigationStore>()
        .and_then(|store| store.records.get_mut(&navigation.get_identity_hash().get()))
    {
        record.current = target;
    }
    fire_success(scope, navigation, from, entry, Some(navigation_type));
}

fn fire_success(
    scope: &mut v8::PinScope<'_, '_>,
    navigation: v8::Local<'_, v8::Object>,
    from: v8::Local<'_, v8::Object>,
    entry: v8::Local<'_, v8::Object>,
    navigation_type: Option<&str>,
) {
    if let Ok(activation) = super::navigation_activation::create(
        scope,
        entry,
        Some(from),
        navigation_type.map(str::to_owned),
    ) {
        let activation = v8::Global::new(scope, activation);
        if let Some(record) = scope
            .get_slot_mut::<NavigationStore>()
            .and_then(|store| store.records.get_mut(&navigation.get_identity_hash().get()))
        {
            record.activation = Some(activation);
            record.transition = None;
        }
    }
    if let Ok(event) = super::navigation_current_entry_change_event::create(
        scope,
        "currententrychange",
        navigation_type.map(str::to_owned),
        from,
    ) {
        fire_event_with_handler(scope, navigation, event, |record| {
            record.oncurrententrychange.clone()
        });
    }
    let event = super::event_target::create_event(scope, "navigatesuccess");
    fire_event_with_handler(scope, navigation, event, |record| {
        record.onnavigatesuccess.clone()
    });
}

fn fire_event_with_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    select: impl FnOnce(&NavigationRecord) -> Option<v8::Global<v8::Value>>,
) {
    super::event_target::dispatch(scope, target, event);
    let handler = record(scope, target).and_then(|record| select(&record));
    if let Some(handler) = handler
        && let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = function.call(scope, target.into(), &[event.into()]);
    }
}

fn resolved_navigation_result<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    entry: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let result = v8::Object::new(scope);
    let committed = super::writable_stream::resolved_promise(scope, entry.into())?;
    let finished = super::writable_stream::resolved_promise(scope, entry.into())?;
    define_data(scope, result, "committed", committed.into());
    define_data(scope, result, "finished", finished.into());
    Ok(result)
}

fn rejected_navigation_result<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let result = v8::Object::new(scope);
    let exception = super::dom_exception::create(
        scope,
        "The requested traversal is unavailable".to_owned(),
        name.to_owned(),
    )?;
    let committed = rejected_promise(scope, exception.into())?;
    let finished = rejected_promise(scope, exception.into())?;
    define_data(scope, result, "committed", committed.into());
    define_data(scope, result, "finished", finished.into());
    Ok(result)
}

fn rejected_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let resolver =
        v8::PromiseResolver::new(scope).ok_or_else(|| "cannot create Promise".to_owned())?;
    if resolver.reject(scope, value) != Some(true) {
        return Err("cannot reject Promise".to_owned());
    }
    Ok(resolver.get_promise(scope))
}

fn resolve_url(base: &str, input: &str) -> String {
    ::url::Url::parse(base)
        .and_then(|base| base.join(input))
        .map(|url| url.to_string())
        .or_else(|_| ::url::Url::parse(input).map(|url| url.to_string()))
        .unwrap_or_else(|_| input.to_owned())
}

fn same_without_fragment(left: &str, right: &str) -> bool {
    let mut left = left.to_owned();
    let mut right = right.to_owned();
    if let Some(index) = left.find('#') {
        left.truncate(index);
    }
    if let Some(index) = right.find('#') {
        right.truncate(index);
    }
    left == right
}

fn value_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    value_property(scope, object, name)
        .filter(|value| !value.is_null_or_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}
