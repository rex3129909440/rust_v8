pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<(v8::Local<'s, v8::Function>, v8::Local<'s, v8::Object>), String> {
    let constructor = crate::webidl::create_function(
        scope,
        "WorkerLocation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::worker_location_href_property::define(scope, prototype)?;
    super::worker_location_origin_property::define(scope, prototype)?;
    super::worker_location_protocol_property::define(scope, prototype)?;
    super::worker_location_host_property::define(scope, prototype)?;
    super::worker_location_hostname_property::define(scope, prototype)?;
    super::worker_location_port_property::define(scope, prototype)?;
    super::worker_location_pathname_property::define(scope, prototype)?;
    super::worker_location_search_property::define(scope, prototype)?;
    super::worker_location_hash_property::define(scope, prototype)?;
    super::worker_location_to_string::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let location = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, location, prototype.into()) != Some(true) {
        return Err("cannot create WorkerLocation".to_owned());
    }
    Ok((constructor, location))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn checked_url(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<url::Url> {
    let Some(record) = super::worker_global_scope::current_record(scope) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return None;
    };
    let valid = record.location.as_ref().is_some_and(|location| {
        v8::Local::new(scope, location).get_identity_hash().get()
            == object.get_identity_hash().get()
    });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return None;
    }
    url::Url::parse(&record.url).ok()
}

fn return_string(scope: &mut v8::PinScope<'_, '_>, result: &mut v8::ReturnValue<'_>, value: &str) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

pub(crate) fn get_href(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = super::worker_global_scope::current_record(scope)
        && checked_url(scope, arguments.this()).is_some()
    {
        return_string(scope, &mut result, &record.url);
    }
}

pub(crate) fn get_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        return_string(scope, &mut result, &url.origin().ascii_serialization());
    }
}

pub(crate) fn get_protocol(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        return_string(scope, &mut result, &format!("{}:", url.scheme()));
    }
}

pub(crate) fn get_host(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        let mut host = url.host_str().unwrap_or_default().to_owned();
        if let Some(port) = url.port() {
            host.push(':');
            host.push_str(&port.to_string());
        }
        return_string(scope, &mut result, &host);
    }
}

pub(crate) fn get_hostname(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        return_string(scope, &mut result, url.host_str().unwrap_or_default());
    }
}

pub(crate) fn get_port(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        return_string(
            scope,
            &mut result,
            &url.port().map(|port| port.to_string()).unwrap_or_default(),
        );
    }
}

pub(crate) fn get_pathname(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        return_string(scope, &mut result, url.path());
    }
}

pub(crate) fn get_search(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        let value = url
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default();
        return_string(scope, &mut result, &value);
    }
}

pub(crate) fn get_hash(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(url) = checked_url(scope, arguments.this()) {
        let value = url
            .fragment()
            .map(|fragment| format!("#{fragment}"))
            .unwrap_or_default();
        return_string(scope, &mut result, &value);
    }
}

pub(crate) fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_href(scope, arguments, result);
}
