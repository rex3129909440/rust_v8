use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub(crate) struct CookieEntry {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) domain: Option<String>,
    pub(crate) path: String,
    pub(crate) expires: Option<f64>,
    pub(crate) secure: bool,
    pub(crate) same_site: String,
    pub(crate) partitioned: bool,
}

#[derive(Default)]
pub(crate) struct CookieStoreStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) instances: HashSet<i32>,
    pub(crate) handlers: HashMap<i32, v8::Global<v8::Value>>,
    pub(crate) entries: Vec<CookieEntry>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CookieStoreStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CookieStore", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CookieStoreStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CookieStore",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::cookie_store_delete::define(scope, prototype)?;
    super::cookie_store_get::define(scope, prototype)?;
    super::cookie_store_get_all::define(scope, prototype)?;
    super::cookie_store_set::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::cookie_store_onchange_property::define(scope, prototype)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CookieStoreStore>()
        .ok_or_else(|| "CookieStore state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CookieStore': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CookieStore".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<CookieStoreStore>()
        .ok_or_else(|| "CookieStore state was not prepared".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

pub(crate) fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<CookieStoreStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}

pub(crate) fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object?.get(scope, key.into())
}

pub(crate) fn text_member(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: &str,
) -> String {
    member(scope, object, name)
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_else(|| default.to_owned())
}

pub(crate) fn requested_name(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<String> {
    if value.is_undefined() {
        return None;
    }
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        return member(scope, Some(object), "name")
            .filter(|value| !value.is_undefined())
            .map(|value| crate::webidl::value_to_string(scope, value));
    }
    Some(crate::webidl::value_to_string(scope, value))
}

pub(crate) fn cookie_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    entry: &CookieEntry,
) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    define_text(scope, object, "name", &entry.name);
    define_text(scope, object, "value", &entry.value);
    match &entry.domain {
        Some(domain) => define_text(scope, object, "domain", domain),
        None => define_value(scope, object, "domain", v8::null(scope).into()),
    }
    define_text(scope, object, "path", &entry.path);
    match entry.expires {
        Some(expires) => define_value(
            scope,
            object,
            "expires",
            v8::Number::new(scope, expires).into(),
        ),
        None => define_value(scope, object, "expires", v8::null(scope).into()),
    }
    define_value(
        scope,
        object,
        "secure",
        v8::Boolean::new(scope, entry.secure).into(),
    );
    define_text(scope, object, "sameSite", &entry.same_site);
    define_value(
        scope,
        object,
        "partitioned",
        v8::Boolean::new(scope, entry.partitioned).into(),
    );
    object
}

pub(crate) fn entry_from_cookie(cookie: &super::document_cookie::Cookie) -> CookieEntry {
    CookieEntry {
        name: cookie.name.clone(),
        value: cookie.value.clone(),
        domain: (!cookie.host_only).then(|| cookie.domain.clone()),
        path: cookie.path.clone(),
        expires: cookie.expires.map(|expires| expires as f64 * 1000.0),
        secure: cookie.secure,
        same_site: cookie.same_site.clone(),
        partitioned: cookie.partitioned,
    }
}

pub(crate) fn resolved(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

pub(crate) fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = requested_name(scope, arguments.get(0));
    let found = super::document_cookie::global_snapshot(scope)
        .into_iter()
        .find(|entry| name.as_ref().is_none_or(|name| entry.name == *name))
        .map(|entry| entry_from_cookie(&entry));
    let value = found
        .as_ref()
        .map(|entry| cookie_object(scope, entry).into())
        .unwrap_or_else(|| v8::null(scope).into());
    resolved(scope, value, result);
}

pub(crate) fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let name = requested_name(scope, arguments.get(0));
    let entries = super::document_cookie::global_snapshot(scope)
        .into_iter()
        .filter(|entry| name.as_ref().is_none_or(|name| entry.name == *name))
        .map(|entry| entry_from_cookie(&entry))
        .collect::<Vec<_>>();
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let value = cookie_object(scope, entry);
        let _ = array.set_index(scope, index as u32, value.into());
    }
    resolved(scope, array.into(), result);
}

pub(crate) fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let name = if options.is_some() {
        text_member(scope, options, "name", "")
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    if name.is_empty() {
        crate::webidl::throw_type_error(scope, "Cookie name cannot be empty");
        return;
    }
    let value = if options.is_some() {
        text_member(scope, options, "value", "")
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    let domain = member(scope, options, "domain")
        .filter(|value| !value.is_null() && !value.is_undefined())
        .map(|value| {
            crate::webidl::value_to_string(scope, value)
                .trim_start_matches('.')
                .to_ascii_lowercase()
        });
    if domain.as_deref().is_some_and(|domain| {
        let host = crate::page_init::host(scope);
        domain != host && !host.ends_with(&format!(".{domain}"))
    }) {
        crate::webidl::throw_type_error(scope, "Cookie domain does not match the page host");
        return;
    }
    let path = text_member(scope, options, "path", "/");
    if !path.starts_with('/') {
        crate::webidl::throw_type_error(scope, "Cookie path must start with '/'");
        return;
    }
    let expires = member(scope, options, "expires").and_then(|value| value.number_value(scope));
    let secure = member(scope, options, "secure").is_some_and(|value| value.boolean_value(scope));
    let same_site = text_member(scope, options, "sameSite", "strict").to_ascii_lowercase();
    if !matches!(same_site.as_str(), "strict" | "lax" | "none") {
        crate::webidl::throw_type_error(scope, "Invalid sameSite value");
        return;
    }
    let partitioned =
        member(scope, options, "partitioned").is_some_and(|value| value.boolean_value(scope));
    let entry = CookieEntry {
        name: name.clone(),
        value: value.clone(),
        domain: domain.clone(),
        path: path.clone(),
        expires,
        secure,
        same_site: same_site.clone(),
        partitioned,
    };
    let cookie = super::document_cookie::Cookie {
        name,
        value,
        domain: domain
            .clone()
            .unwrap_or_else(|| crate::page_init::host(scope)),
        host_only: domain.is_none(),
        path,
        expires: expires.map(|expires| (expires / 1000.0) as i64),
        secure,
        same_site,
        partitioned,
    };
    let now = crate::determinism::date_epoch_milliseconds(scope);
    if expires.is_some_and(|expires| expires <= now) {
        let deleted = super::document_cookie::delete_from_cookie_store(
            scope,
            &cookie.name,
            domain.as_deref(),
            Some(&cookie.path),
        )
        .map(|cookie| entry_from_cookie(&cookie));
        if deleted.is_some() {
            notify(scope, arguments.this(), None, deleted);
        }
        resolved(scope, v8::undefined(scope).into(), result);
        return;
    }
    let _old = super::document_cookie::set_from_cookie_store(scope, cookie);
    notify(scope, arguments.this(), Some(entry), None);
    resolved(scope, v8::undefined(scope).into(), result);
}

pub(crate) fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let name = requested_name(scope, arguments.get(0)).unwrap_or_default();
    let domain = member(scope, options, "domain")
        .filter(|value| !value.is_null() && !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value));
    let path = member(scope, options, "path")
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value));
    let removed = super::document_cookie::delete_from_cookie_store(
        scope,
        &name,
        domain.as_deref(),
        path.as_deref(),
    )
    .map(|cookie| entry_from_cookie(&cookie));
    notify(scope, arguments.this(), None, removed);
    resolved(scope, v8::undefined(scope).into(), result);
}

pub(crate) fn notify(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    changed: Option<CookieEntry>,
    deleted: Option<CookieEntry>,
) {
    let changed_array = v8::Array::new(scope, usize::from(changed.is_some()) as i32);
    if let Some(changed) = changed {
        let value = cookie_object(scope, &changed);
        let _ = changed_array.set_index(scope, 0, value.into());
    }
    let deleted_array = v8::Array::new(scope, usize::from(deleted.is_some()) as i32);
    if let Some(deleted) = deleted {
        let value = cookie_object(scope, &deleted);
        let _ = deleted_array.set_index(scope, 0, value.into());
    }
    if let Ok(event) = super::cookie_change_event::create(scope, changed_array, deleted_array) {
        let _ = super::event_target::dispatch(scope, target, event);
    }
}

pub(crate) fn notify_cookie_mutation(
    scope: &mut v8::PinScope<'_, '_>,
    changed: Option<super::document_cookie::Cookie>,
    deleted: Option<super::document_cookie::Cookie>,
) {
    let Some(target) = super::cookie_store_global::value(scope) else {
        return;
    };
    notify(
        scope,
        target,
        changed.as_ref().map(entry_from_cookie),
        deleted.as_ref().map(entry_from_cookie),
    );
}

pub(crate) fn get_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let handler = scope
        .get_slot::<CookieStoreStore>()
        .and_then(|store| {
            store
                .handlers
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    super::window_event_handler_support::return_handler(scope, handler, result);
}

pub(crate) fn set_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let identity = arguments.this().get_identity_hash().get();
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<CookieStoreStore>() {
        match handler {
            Some(handler) => {
                store.handlers.insert(identity, handler);
            }
            None => {
                store.handlers.remove(&identity);
            }
        }
    }
    let present = scope
        .get_slot::<CookieStoreStore>()
        .is_some_and(|store| store.handlers.contains_key(&identity));
    super::event_target::set_attribute_handler(scope, arguments.this(), "change", present);
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "change" || !valid(scope, target) {
        return;
    }
    let handler = scope
        .get_slot::<CookieStoreStore>()
        .and_then(|store| store.handlers.get(&target.get_identity_hash().get()))
        .cloned();
    if let Some(handler) = handler
        && let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = handler.call(scope, target.into(), &[event.into()]);
    }
}

pub(crate) fn define_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define_value(scope, object, name, value.into());
    }
}

pub(crate) fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::NONE);
    }
}
