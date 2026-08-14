use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HistoryStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, HistoryRecord>,
}

#[derive(Clone)]
struct HistoryEntry {
    state: v8::Global<v8::Value>,
    title: String,
    url: String,
}

#[derive(Clone)]
struct HistoryRecord {
    entries: Vec<HistoryEntry>,
    current: usize,
    scroll_restoration: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HistoryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "History", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<HistoryStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "History",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "scrollRestoration",
        get_scroll_restoration,
        set_scroll_restoration,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_method(scope, prototype, "back", 0, back)?;
    crate::webidl::define_method(scope, prototype, "forward", 0, forward)?;
    crate::webidl::define_method(scope, prototype, "go", 0, go)?;
    crate::webidl::define_method(scope, prototype, "pushState", 2, push_state)?;
    crate::webidl::define_method(scope, prototype, "replaceState", 2, replace_state)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HistoryStore>()
        .ok_or_else(|| "History state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create History".to_owned());
    }
    let initial_state: v8::Local<v8::Value> = v8::null(scope).into();
    let initial_state = v8::Global::new(scope, initial_state);
    scope
        .get_slot_mut::<HistoryStore>()
        .ok_or_else(|| "History state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            HistoryRecord {
                entries: vec![HistoryEntry {
                    state: initial_state,
                    title: String::new(),
                    url: String::new(),
                }],
                current: 0,
                scroll_restoration: "auto".to_owned(),
            },
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<HistoryRecord> {
    scope
        .get_slot::<HistoryStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.entries.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_scroll_restoration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.scroll_restoration) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_scroll_restoration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "auto" && value != "manual" {
        crate::webidl::throw_type_error(scope, "Invalid scroll restoration mode");
        return;
    }
    if let Some(record) = scope.get_slot_mut::<HistoryStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.scroll_restoration = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, &record.entries[record.current].state));
}

fn back(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    move_by(scope, arguments.this(), -1);
}

fn forward(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    move_by(scope, arguments.this(), 1);
}

fn go(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let delta = arguments.get(0).int32_value(scope).unwrap_or(0);
    move_by(scope, arguments.this(), delta);
}

fn move_by(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, delta: i32) {
    if let Some(record) = scope
        .get_slot_mut::<HistoryStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        let target = record.current as i64 + i64::from(delta);
        if target >= 0 && target < record.entries.len() as i64 {
            record.current = target as usize;
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn push_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(state) = clone_history_state(scope, arguments.get(0), "pushState") else {
        return;
    };
    let title = crate::webidl::value_to_string(scope, arguments.get(1));
    let Some(url) = history_url(scope, arguments.get(2), "pushState") else {
        return;
    };
    if let Some(record) = scope.get_slot_mut::<HistoryStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.entries.truncate(record.current + 1);
        record.entries.push(HistoryEntry {
            state,
            title,
            url: url.clone(),
        });
        record.current = record.entries.len() - 1;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    apply_history_url(scope, &url);
}

fn replace_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(state) = clone_history_state(scope, arguments.get(0), "replaceState") else {
        return;
    };
    let title = crate::webidl::value_to_string(scope, arguments.get(1));
    let Some(url) = history_url(scope, arguments.get(2), "replaceState") else {
        return;
    };
    if let Some(record) = scope.get_slot_mut::<HistoryStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.entries[record.current] = HistoryEntry {
            state,
            title,
            url: url.clone(),
        };
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    apply_history_url(scope, &url);
}

fn clone_history_state(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    method: &str,
) -> Option<v8::Global<v8::Value>> {
    let description = if let Ok(symbol) = v8::Local::<v8::Symbol>::try_from(value) {
        let description = symbol.description(scope).to_rust_string_lossy(scope);
        format!("Symbol({description})")
    } else {
        crate::webidl::value_to_string(scope, value)
    };
    let context = v8::Global::new(scope, scope.get_entered_or_microtask_context());
    let context = v8::Local::new(scope, &context);
    match super::structured_clone::clone_into(
        scope,
        context,
        value,
        super::structured_clone::TransferList::default(),
    ) {
        Ok(output) => Some(output.value),
        Err(_) => {
            super::structured_clone::throw_data_clone_error(
                scope,
                &format!(
                    "Failed to execute '{method}' on 'History': {description} could not be cloned."
                ),
            );
            None
        }
    }
}

fn history_url(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    method: &str,
) -> Option<String> {
    let location = super::location_global::value(scope)?;
    let current = super::location::current_url(scope, location)?;
    if value.is_undefined() {
        return Some(current.as_str().to_owned());
    }
    let requested = crate::webidl::value_to_string(scope, value);
    let Ok(resolved) = ::url::Url::parse(&requested).or_else(|_| current.join(&requested)) else {
        super::node::throw_dom_exception(
            scope,
            "SecurityError",
            &format!(
                "Failed to execute '{method}' on 'History': A history state object with URL '{requested}' cannot be created in a document with origin '{}' and URL '{}'.",
                current.origin().ascii_serialization(),
                current.as_str()
            ),
        );
        return None;
    };
    if resolved.origin() != current.origin() {
        super::node::throw_dom_exception(
            scope,
            "SecurityError",
            &format!(
                "Failed to execute '{method}' on 'History': A history state object with URL '{}' cannot be created in a document with origin '{}' and URL '{}'.",
                resolved.as_str(),
                current.origin().ascii_serialization(),
                current.as_str()
            ),
        );
        return None;
    }
    Some(resolved.as_str().to_owned())
}

fn apply_history_url(scope: &mut v8::PinScope<'_, '_>, url: &str) {
    let Ok(parsed) = ::url::Url::parse(url) else {
        return;
    };
    if let Some(location) = super::location_global::value(scope) {
        let _ = super::location::replace_url(scope, location, parsed);
    }
    if let Some(document) = super::document_global::value(scope) {
        super::document::set_string_value(scope, document, "URL", url);
        super::document::set_string_value(scope, document, "documentURI", url);
    }
}
