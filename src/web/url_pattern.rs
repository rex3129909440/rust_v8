use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct UrlPatternStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, PatternRecord>,
}

#[derive(Clone)]
struct PatternRecord {
    protocol: String,
    username: String,
    password: String,
    hostname: String,
    port: String,
    pathname: String,
    search: String,
    hash: String,
    has_regexp_groups: bool,
}

#[derive(Clone)]
struct UrlComponents {
    input: String,
    protocol: String,
    username: String,
    password: String,
    hostname: String,
    port: String,
    pathname: String,
    search: String,
    hash: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(UrlPatternStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "URLPattern", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<UrlPatternStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "URLPattern",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "protocol", get_protocol)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "username", get_username)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "password", get_password)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "hostname", get_hostname)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "port", get_port)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pathname", get_pathname)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "search", get_search)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "hash", get_hash)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "hasRegExpGroups",
        get_has_regexp_groups,
    )?;
    crate::webidl::define_method(scope, prototype, "exec", 0, exec)?;
    crate::webidl::define_method(scope, prototype, "test", 0, test)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<UrlPatternStore>()
        .ok_or_else(|| "URLPattern state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "URLPattern must be constructed with new");
        return;
    }
    let base = if arguments.get(1).is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, arguments.get(1)))
    };
    let pattern = if let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        pattern_from_object(scope, init, base.as_deref())
    } else {
        let input = if arguments.get(0).is_undefined() {
            "*".to_owned()
        } else {
            crate::webidl::value_to_string(scope, arguments.get(0))
        };
        pattern_from_string(&input, base.as_deref())
    };
    let Ok(pattern) = pattern else {
        crate::webidl::throw_type_error(scope, "Invalid URLPattern");
        return;
    };
    if let Some(store) = scope.get_slot_mut::<UrlPatternStore>() {
        store
            .records
            .insert(arguments.this().get_identity_hash().get(), pattern);
    }
    result.set(arguments.this().into());
}

fn pattern_from_string(input: &str, base: Option<&str>) -> Result<PatternRecord, String> {
    if input == "*" {
        return Ok(wildcard_pattern());
    }
    if input.contains("://") && (input.contains('*') || input.contains(':')) {
        return Ok(split_pattern_url(input));
    }
    let parsed = if let Ok(url) = url::Url::parse(input) {
        url
    } else {
        let base = base.ok_or_else(|| "relative pattern requires a base URL".to_owned())?;
        url::Url::parse(base)
            .map_err(|error| error.to_string())?
            .join(input)
            .map_err(|error| error.to_string())?
    };
    Ok(pattern_from_url(&parsed))
}

fn split_pattern_url(input: &str) -> PatternRecord {
    let (protocol, rest) = input.split_once("://").unwrap_or(("*", input));
    let (authority, path_and_more) = rest.split_once('/').unwrap_or((rest, "*"));
    let (hostname, port) = authority
        .rsplit_once(':')
        .filter(|(_, port)| !port.contains(']'))
        .unwrap_or((authority, "*"));
    let (path_and_search, hash) = path_and_more
        .split_once('#')
        .unwrap_or((path_and_more, "*"));
    let (pathname, search) = path_and_search
        .split_once('?')
        .unwrap_or((path_and_search, "*"));
    let pathname = format!("/{pathname}");
    let has_regexp_groups = protocol.contains('(')
        || hostname.contains('(')
        || port.contains('(')
        || pathname.contains('(')
        || search.contains('(')
        || hash.contains('(');
    PatternRecord {
        protocol: protocol.trim_end_matches(':').to_owned(),
        username: "*".to_owned(),
        password: "*".to_owned(),
        hostname: hostname.to_owned(),
        port: port.to_owned(),
        pathname,
        search: search.to_owned(),
        hash: hash.to_owned(),
        has_regexp_groups,
    }
}

fn wildcard_pattern() -> PatternRecord {
    PatternRecord {
        protocol: "*".to_owned(),
        username: "*".to_owned(),
        password: "*".to_owned(),
        hostname: "*".to_owned(),
        port: "*".to_owned(),
        pathname: "*".to_owned(),
        search: "*".to_owned(),
        hash: "*".to_owned(),
        has_regexp_groups: false,
    }
}

fn pattern_from_url(url: &url::Url) -> PatternRecord {
    PatternRecord {
        protocol: url.scheme().to_owned(),
        username: url.username().to_owned(),
        password: url.password().unwrap_or_default().to_owned(),
        hostname: url.host_str().unwrap_or_default().to_owned(),
        port: url.port().map(|port| port.to_string()).unwrap_or_default(),
        pathname: url.path().to_owned(),
        search: url.query().unwrap_or_default().to_owned(),
        hash: url.fragment().unwrap_or_default().to_owned(),
        has_regexp_groups: false,
    }
}

fn pattern_from_object(
    scope: &v8::PinScope<'_, '_>,
    init: v8::Local<'_, v8::Object>,
    base: Option<&str>,
) -> Result<PatternRecord, String> {
    let object_base = string_property(scope, init, "baseURL");
    let base = object_base.as_deref().or(base);
    let mut pattern = if let Some(base) = base {
        url::Url::parse(base)
            .map(|url| pattern_from_url(&url))
            .map_err(|error| error.to_string())?
    } else {
        wildcard_pattern()
    };
    if let Some(value) = string_property(scope, init, "protocol") {
        pattern.protocol = value.trim_end_matches(':').to_owned();
    }
    if let Some(value) = string_property(scope, init, "username") {
        pattern.username = value;
    }
    if let Some(value) = string_property(scope, init, "password") {
        pattern.password = value;
    }
    if let Some(value) = string_property(scope, init, "hostname") {
        pattern.hostname = value;
    }
    if let Some(value) = string_property(scope, init, "port") {
        pattern.port = value;
    }
    if let Some(value) = string_property(scope, init, "pathname") {
        pattern.pathname = value;
    }
    if let Some(value) = string_property(scope, init, "search") {
        pattern.search = value.trim_start_matches('?').to_owned();
    }
    if let Some(value) = string_property(scope, init, "hash") {
        pattern.hash = value.trim_start_matches('#').to_owned();
    }
    pattern.has_regexp_groups = pattern.protocol.contains('(')
        || pattern.username.contains('(')
        || pattern.password.contains('(')
        || pattern.hostname.contains('(')
        || pattern.port.contains('(')
        || pattern.pathname.contains('(')
        || pattern.search.contains('(')
        || pattern.hash.contains('(');
    Ok(pattern)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PatternRecord> {
    scope
        .get_slot::<UrlPatternStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PatternRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.protocol)
}
fn get_username(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.username)
}
fn get_password(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.password)
}
fn get_hostname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.hostname)
}
fn get_port(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.port)
}
fn get_pathname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.pathname)
}
fn get_search(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.search)
}
fn get_hash(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.hash)
}

fn get_has_regexp_groups(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.has_regexp_groups).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn input_components(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    base_value: v8::Local<'_, v8::Value>,
) -> Result<UrlComponents, String> {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        return Ok(UrlComponents {
            input: "[object Object]".to_owned(),
            protocol: string_property(scope, object, "protocol").unwrap_or_default(),
            username: string_property(scope, object, "username").unwrap_or_default(),
            password: string_property(scope, object, "password").unwrap_or_default(),
            hostname: string_property(scope, object, "hostname").unwrap_or_default(),
            port: string_property(scope, object, "port").unwrap_or_default(),
            pathname: string_property(scope, object, "pathname").unwrap_or_default(),
            search: string_property(scope, object, "search")
                .unwrap_or_default()
                .trim_start_matches('?')
                .to_owned(),
            hash: string_property(scope, object, "hash")
                .unwrap_or_default()
                .trim_start_matches('#')
                .to_owned(),
        });
    }
    let input = crate::webidl::value_to_string(scope, value);
    let parsed = if let Ok(url) = url::Url::parse(&input) {
        url
    } else {
        if base_value.is_undefined() {
            return Err("relative URL requires a base URL".to_owned());
        }
        let base = crate::webidl::value_to_string(scope, base_value);
        url::Url::parse(&base)
            .map_err(|error| error.to_string())?
            .join(&input)
            .map_err(|error| error.to_string())?
    };
    Ok(UrlComponents {
        input,
        protocol: parsed.scheme().to_owned(),
        username: parsed.username().to_owned(),
        password: parsed.password().unwrap_or_default().to_owned(),
        hostname: parsed.host_str().unwrap_or_default().to_owned(),
        port: parsed
            .port()
            .map(|port| port.to_string())
            .unwrap_or_default(),
        pathname: parsed.path().to_owned(),
        search: parsed.query().unwrap_or_default().to_owned(),
        hash: parsed.fragment().unwrap_or_default().to_owned(),
    })
}

fn component_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if pattern.starts_with(':') {
        return !value.is_empty();
    }
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        return value.starts_with(prefix) && value.ends_with(suffix);
    }
    if pattern.contains("/:") {
        let pattern_parts = pattern.split('/').collect::<Vec<_>>();
        let value_parts = value.split('/').collect::<Vec<_>>();
        return pattern_parts.len() == value_parts.len()
            && pattern_parts
                .iter()
                .zip(value_parts.iter())
                .all(|(pattern, value)| pattern.starts_with(':') || pattern == value);
    }
    pattern == value
}

fn matches(record: &PatternRecord, input: &UrlComponents) -> bool {
    component_matches(&record.protocol, &input.protocol)
        && component_matches(&record.username, &input.username)
        && component_matches(&record.password, &input.password)
        && component_matches(&record.hostname, &input.hostname)
        && component_matches(&record.port, &input.port)
        && component_matches(&record.pathname, &input.pathname)
        && component_matches(&record.search, &input.search)
        && component_matches(&record.hash, &input.hash)
}

fn test(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let matched = input_components(scope, arguments.get(0), arguments.get(1))
        .is_ok_and(|input| matches(&record, &input));
    result.set(v8::Boolean::new(scope, matched).into());
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

fn component_result<'s>(scope: &v8::PinScope<'s, '_>, value: &str) -> v8::Local<'s, v8::Object> {
    let output = v8::Object::new(scope);
    if let Some(value) = v8::String::new(scope, value) {
        define_data(scope, output, "input", value.into());
    }
    define_data(scope, output, "groups", v8::Object::new(scope).into());
    output
}

fn exec(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(input) = input_components(scope, arguments.get(0), arguments.get(1)) else {
        result.set(v8::null(scope).into());
        return;
    };
    if !matches(&record, &input) {
        result.set(v8::null(scope).into());
        return;
    }
    let output = v8::Object::new(scope);
    let inputs = v8::Array::new(scope, 1);
    if let Some(value) = v8::String::new(scope, &input.input) {
        let _ = inputs.set_index(scope, 0, value.into());
    }
    define_data(scope, output, "inputs", inputs.into());
    let protocol = component_result(scope, &input.protocol);
    define_data(scope, output, "protocol", protocol.into());
    let username = component_result(scope, &input.username);
    define_data(scope, output, "username", username.into());
    let password = component_result(scope, &input.password);
    define_data(scope, output, "password", password.into());
    let hostname = component_result(scope, &input.hostname);
    define_data(scope, output, "hostname", hostname.into());
    let port = component_result(scope, &input.port);
    define_data(scope, output, "port", port.into());
    let pathname = component_result(scope, &input.pathname);
    define_data(scope, output, "pathname", pathname.into());
    let search = component_result(scope, &input.search);
    define_data(scope, output, "search", search.into());
    let hash = component_result(scope, &input.hash);
    define_data(scope, output, "hash", hash.into());
    result.set(output.into());
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then(|| crate::webidl::value_to_string(scope, value))
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UrlPatternStore>() {
        store.constructors.remove(&realm_id);
    }
}
