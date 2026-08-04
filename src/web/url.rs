use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct UrlStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, UrlRecord>,
    object_urls: HashMap<String, (Vec<u8>, String)>,
    next_object_url: u64,
}

#[derive(Clone)]
struct UrlRecord {
    parsed: url::Url,
    search_params: v8::Global<v8::Object>,
    search_params_id: i32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(UrlStore::default());
}

#[allow(dead_code)]
pub(crate) fn install_standard_name(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "URL", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<UrlStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }

    super::url_search_params::ensure_constructor(scope)?;
    let constructor =
        crate::webidl::create_function(scope, "URL", 1, v8::ConstructorBehavior::Allow, construct)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;

    crate::webidl::define_readonly_accessor(scope, prototype, "origin", get_origin)?;
    crate::webidl::define_accessor(scope, prototype, "protocol", get_protocol, set_protocol)?;
    crate::webidl::define_accessor(scope, prototype, "username", get_username, set_username)?;
    crate::webidl::define_accessor(scope, prototype, "password", get_password, set_password)?;
    crate::webidl::define_accessor(scope, prototype, "host", get_host, set_host)?;
    crate::webidl::define_accessor(scope, prototype, "hostname", get_hostname, set_hostname)?;
    crate::webidl::define_accessor(scope, prototype, "port", get_port, set_port)?;
    crate::webidl::define_accessor(scope, prototype, "pathname", get_pathname, set_pathname)?;
    crate::webidl::define_accessor(scope, prototype, "search", get_search, set_search)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "searchParams", get_search_params)?;
    crate::webidl::define_accessor(scope, prototype, "hash", get_hash, set_hash)?;
    crate::webidl::define_accessor(scope, prototype, "href", get_href, set_href)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;

    crate::webidl::define_method(scope, constructor.into(), "canParse", 1, can_parse)?;
    crate::webidl::define_method(scope, constructor.into(), "parse", 1, parse)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "createObjectURL",
        1,
        create_object_url,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "revokeObjectURL",
        1,
        revoke_object_url,
    )?;

    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<UrlStore>()
        .ok_or_else(|| "URL state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'URL': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "Failed to construct 'URL': 1 argument required");
        return;
    }
    let parsed = match parse_arguments(scope, &arguments) {
        Ok(parsed) => parsed,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let object = arguments.this();
    if let Err(message) = attach_record(scope, object, parsed) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    result.set(object.into());
}

fn parse_arguments(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Result<url::Url, String> {
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    if arguments.length() > 1 && !arguments.get(1).is_undefined() {
        let base = crate::webidl::value_to_string(scope, arguments.get(1));
        let base = url::Url::parse(&base).map_err(|_| "Invalid base URL".to_owned())?;
        base.join(&input).map_err(|_| "Invalid URL".to_owned())
    } else {
        url::Url::parse(&input).map_err(|_| "Invalid URL".to_owned())
    }
}

fn attach_record(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    parsed: url::Url,
) -> Result<(), String> {
    let id = object.get_identity_hash().get();
    let query = parsed.query().unwrap_or_default();
    let (search_params, search_params_id) =
        super::url_search_params::create_linked(scope, query, id)?;
    let record = UrlRecord {
        parsed,
        search_params: v8::Global::new(scope, search_params),
        search_params_id,
    };
    scope
        .get_slot_mut::<UrlStore>()
        .ok_or_else(|| "URL state is missing".to_owned())?
        .records
        .insert(id, record);
    Ok(())
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<UrlRecord> {
    scope
        .get_slot::<UrlStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut url::Url),
) {
    let id = object.get_identity_hash().get();
    let sync = if let Some(record) = scope
        .get_slot_mut::<UrlStore>()
        .and_then(|store| store.records.get_mut(&id))
    {
        operation(&mut record.parsed);
        Some((
            record.search_params_id,
            record.parsed.query().unwrap_or_default().to_owned(),
        ))
    } else {
        None
    };
    if let Some((search_params_id, query)) = sync {
        super::url_search_params::replace_query(scope, search_params_id, &query);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation: receiver is not a URL");
    }
}

pub(super) fn set_query_from_params(scope: &mut v8::PinScope<'_, '_>, url_id: i32, query: &str) {
    if let Some(record) = scope
        .get_slot_mut::<UrlStore>()
        .and_then(|store| store.records.get_mut(&url_id))
    {
        record
            .parsed
            .set_query(if query.is_empty() { None } else { Some(query) });
    }
}

fn return_string(scope: &mut v8::PinScope<'_, '_>, result: &mut v8::ReturnValue<'_>, value: &str) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn require_record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<UrlRecord> {
    let value = record(scope, object);
    if value.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    value
}

fn get_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(
            scope,
            &mut result,
            &record.parsed.origin().ascii_serialization(),
        );
    }
}

fn get_protocol(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(scope, &mut result, &format!("{}:", record.parsed.scheme()));
    }
}

fn set_protocol(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let scheme = crate::webidl::value_to_string(scope, arguments.get(0));
    let scheme = scheme.trim_end_matches(':').to_owned();
    update(scope, arguments.this(), |url| {
        let _ = url.set_scheme(&scheme);
    });
}

fn get_username(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(scope, &mut result, record.parsed.username());
    }
}

fn set_username(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |url| {
        let _ = url.set_username(&value);
    });
}

fn get_password(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(
            scope,
            &mut result,
            record.parsed.password().unwrap_or_default(),
        );
    }
}

fn set_password(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |url| {
        let _ = url.set_password(if value.is_empty() { None } else { Some(&value) });
    });
}

fn get_host(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        let host = record.parsed.host_str().unwrap_or_default();
        let value = record
            .parsed
            .port()
            .map(|port| format!("{host}:{port}"))
            .unwrap_or_else(|| host.to_owned());
        return_string(scope, &mut result, &value);
    }
}

fn set_host(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |url| {
        if let Ok(candidate) = url::Url::parse(&format!("{}://{value}/", url.scheme())) {
            let _ = url.set_host(candidate.host_str());
            let _ = url.set_port(candidate.port());
        }
    });
}

fn get_hostname(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(
            scope,
            &mut result,
            record.parsed.host_str().unwrap_or_default(),
        );
    }
}

fn set_hostname(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |url| {
        let _ = url.set_host(if value.is_empty() { None } else { Some(&value) });
    });
}

fn get_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        let value = record
            .parsed
            .port()
            .map(|port| port.to_string())
            .unwrap_or_default();
        return_string(scope, &mut result, &value);
    }
}

fn set_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |url| {
        let port = if value.is_empty() {
            None
        } else if let Ok(port) = value.parse::<u16>() {
            Some(port)
        } else {
            return;
        };
        let _ = url.set_port(port);
    });
}

fn get_pathname(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(scope, &mut result, record.parsed.path());
    }
}

fn set_pathname(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |url| url.set_path(&value));
}

fn get_search(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        let value = record
            .parsed
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default();
        return_string(scope, &mut result, &value);
    }
}

fn set_search(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = value.strip_prefix('?').unwrap_or(&value).to_owned();
    update(scope, arguments.this(), |url| {
        url.set_query(if value.is_empty() { None } else { Some(&value) })
    });
}

fn get_search_params(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        let object = v8::Local::new(scope, &record.search_params);
        result.set(object.into());
    }
}

fn get_hash(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        let value = record
            .parsed
            .fragment()
            .map(|fragment| format!("#{fragment}"))
            .unwrap_or_default();
        return_string(scope, &mut result, &value);
    }
}

fn set_hash(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = value.strip_prefix('#').unwrap_or(&value).to_owned();
    update(scope, arguments.this(), |url| {
        url.set_fragment(if value.is_empty() { None } else { Some(&value) })
    });
}

fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(scope, &mut result, record.parsed.as_str());
    }
}

fn set_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(parsed) = url::Url::parse(&value) else {
        crate::webidl::throw_type_error(scope, "Invalid URL");
        return;
    };
    update(scope, arguments.this(), |url| *url = parsed);
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(scope, &mut result, record.parsed.as_str());
    }
}

fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = require_record(scope, arguments.this()) {
        return_string(scope, &mut result, record.parsed.as_str());
    }
}

fn can_parse(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "URL.canParse requires 1 argument");
        return;
    }
    result.set(v8::Boolean::new(scope, parse_arguments(scope, &arguments).is_ok()).into());
}

fn parse(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "URL.parse requires 1 argument");
        return;
    }
    let Ok(parsed) = parse_arguments(scope, &arguments) else {
        result.set(v8::null(scope).into());
        return;
    };
    let constructor = scope
        .get_slot::<UrlStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    let Some(constructor) = constructor else {
        crate::webidl::throw_type_error(scope, "URL is not initialized");
        return;
    };
    let constructor = v8::Local::new(scope, &constructor);
    let Ok(prototype) = crate::webidl::prototype(scope, constructor) else {
        crate::webidl::throw_type_error(scope, "URL prototype is missing");
        return;
    };
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        crate::webidl::throw_type_error(scope, "Cannot create URL");
        return;
    }
    if let Err(message) = attach_record(scope, object, parsed) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    result.set(object.into());
}

fn create_object_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 || !arguments.get(0).is_object() {
        crate::webidl::throw_type_error(
            scope,
            "URL.createObjectURL requires a Blob or MediaSource",
        );
        return;
    }
    let snapshot = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .and_then(|object| super::blob::byte_snapshot(scope, object))
        .unwrap_or_else(|| (Vec::new(), String::new()));
    let origin = super::worker_global_scope::current_origin(scope).unwrap_or_else(|| {
        let global = scope.get_current_context().global(scope);
        super::html_i_frame_element::origin_for_window(scope, global)
    });
    let mut uuid = [0_u8; 16];
    if !super::crypto::fill_random(scope, &mut uuid) {
        crate::webidl::throw_type_error(scope, "The system random generator failed");
        return;
    }
    uuid[6] = (uuid[6] & 0x0f) | 0x40;
    uuid[8] = (uuid[8] & 0x3f) | 0x80;
    let identifier = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        uuid[0],
        uuid[1],
        uuid[2],
        uuid[3],
        uuid[4],
        uuid[5],
        uuid[6],
        uuid[7],
        uuid[8],
        uuid[9],
        uuid[10],
        uuid[11],
        uuid[12],
        uuid[13],
        uuid[14],
        uuid[15],
    );
    let store = scope.get_slot_mut::<UrlStore>().expect("URL state");
    store.next_object_url += 1;
    let value = format!("blob:{origin}/{identifier}");
    store.object_urls.insert(value.clone(), snapshot);
    return_string(scope, &mut result, &value);
}

fn revoke_object_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "URL.revokeObjectURL requires 1 argument");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    scope
        .get_slot_mut::<UrlStore>()
        .expect("URL state")
        .object_urls
        .remove(&value);
}

pub(crate) fn object_url_snapshot(
    scope: &v8::PinScope<'_, '_>,
    value: &str,
) -> Option<(Vec<u8>, String)> {
    scope
        .get_slot::<UrlStore>()?
        .object_urls
        .get(value)
        .cloned()
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UrlStore>() {
        store.constructors.remove(&realm_id);
    }
}
