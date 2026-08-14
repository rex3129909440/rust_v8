use super::cookie_store::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "set", 1, set)
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "CookieStore", "set", result);
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
