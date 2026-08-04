use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct Cookie {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) domain: String,
    pub(crate) host_only: bool,
    pub(crate) path: String,
    pub(crate) expires: Option<i64>,
    pub(crate) secure: bool,
    pub(crate) same_site: String,
    pub(crate) partitioned: bool,
}

#[derive(Default)]
pub(crate) struct DocumentCookieStore {
    jars: HashMap<i32, Vec<Cookie>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentCookieStore::default());
}

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "cookie", get_cookie, set_cookie)
}

fn valid_document(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    super::node::record(scope, object).is_some_and(|record| record.node_type == 9)
}

fn now_seconds(scope: &v8::PinScope<'_, '_>) -> i64 {
    (crate::determinism::date_epoch_milliseconds(scope) / 1_000.0) as i64
}

fn get_cookie(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let identity = arguments.this().get_identity_hash().get();
    let now = now_seconds(scope);
    let host = crate::page_init::host(scope);
    let request_path = crate::page_init::path(scope);
    let value = scope
        .get_slot_mut::<DocumentCookieStore>()
        .map(|store| {
            let cookies = store.jars.entry(identity).or_default();
            cookies.retain(|cookie| cookie.expires.is_none_or(|expires| expires > now));
            cookies
                .iter()
                .filter(|cookie| {
                    domain_matches(&host, &cookie.domain, cookie.host_only)
                        && path_matches(&request_path, &cookie.path)
                })
                .map(|cookie| format!("{}={}", cookie.name, cookie.value))
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_cookie(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let host = crate::page_init::host(scope);
    let default_path = default_cookie_path(&crate::page_init::path(scope));
    let now = now_seconds(scope);
    let Some(parsed) = parse_set_cookie(&source, &host, &default_path, now) else {
        return;
    };
    let identity = arguments.this().get_identity_hash().get();
    let mut changed = None;
    let mut deleted = None;
    if let Some(store) = scope.get_slot_mut::<DocumentCookieStore>() {
        let cookies = store.jars.entry(identity).or_default();
        cookies.retain(|cookie| cookie.expires.is_none_or(|expires| expires > now));
        let existing = cookies
            .iter()
            .find(|cookie| {
                cookie.name == parsed.cookie.name
                    && cookie.domain == parsed.cookie.domain
                    && cookie.path == parsed.cookie.path
            })
            .cloned();
        cookies.retain(|cookie| {
            cookie.name != parsed.cookie.name
                || cookie.domain != parsed.cookie.domain
                || cookie.path != parsed.cookie.path
        });
        if parsed.delete {
            deleted = existing.or_else(|| Some(parsed.cookie.clone()));
        } else {
            changed = Some(parsed.cookie.clone());
            cookies.push(parsed.cookie);
        }
    }
    super::cookie_store::notify_cookie_mutation(scope, changed, deleted);
}

struct ParsedCookie {
    cookie: Cookie,
    delete: bool,
}

fn parse_set_cookie(
    source: &str,
    host: &str,
    default_path: &str,
    now: i64,
) -> Option<ParsedCookie> {
    let mut parts = source.split(';');
    let name_value = parts.next()?.trim();
    let (name, value) = name_value.split_once('=')?;
    let name = name.trim();
    let value = value.trim();
    if name.is_empty()
        || !name.bytes().all(valid_cookie_name_byte)
        || !value.bytes().all(valid_cookie_value_byte)
    {
        return None;
    }

    let mut domain = host.to_owned();
    let mut host_only = true;
    let mut path = default_path.to_owned();
    let mut expires = None;
    let mut max_age = None;
    let mut secure = false;
    let mut saw_domain = false;

    for attribute in parts {
        let attribute = attribute.trim();
        if attribute.is_empty() {
            continue;
        }
        let (raw_name, raw_value) = attribute
            .split_once('=')
            .map_or((attribute, ""), |(name, value)| (name, value.trim()));
        match raw_name.trim().to_ascii_lowercase().as_str() {
            "domain" => {
                let candidate = raw_value
                    .trim_start_matches('.')
                    .trim_end_matches('.')
                    .to_ascii_lowercase();
                if candidate.is_empty() || !domain_matches(host, &candidate, false) {
                    return None;
                }
                domain = candidate;
                host_only = false;
                saw_domain = true;
            }
            "path" => {
                if raw_value.starts_with('/') {
                    path = raw_value.to_owned();
                }
            }
            "max-age" => {
                if let Ok(seconds) = raw_value.parse::<i64>() {
                    max_age = Some(seconds);
                }
            }
            "expires" => expires = parse_cookie_date(raw_value),
            "secure" => secure = true,
            _ => {}
        }
    }

    if name.starts_with("__Secure-") && !secure {
        return None;
    }
    if name.starts_with("__Host-") && (!secure || saw_domain || path != "/") {
        return None;
    }

    let expires = match max_age {
        Some(seconds) if seconds <= 0 => Some(0),
        Some(seconds) => Some(now.saturating_add(seconds)),
        None => expires,
    };
    let delete = expires.is_some_and(|timestamp| timestamp <= now);
    Some(ParsedCookie {
        cookie: Cookie {
            name: name.to_owned(),
            value: value.to_owned(),
            domain,
            host_only,
            path,
            expires,
            secure,
            same_site: "lax".to_owned(),
            partitioned: false,
        },
        delete,
    })
}

pub(crate) fn global_snapshot(scope: &mut v8::PinScope<'_, '_>) -> Vec<Cookie> {
    let Some(document) = super::document_global::value(scope) else {
        return Vec::new();
    };
    let identity = document.get_identity_hash().get();
    let now = now_seconds(scope);
    scope
        .get_slot_mut::<DocumentCookieStore>()
        .map(|store| {
            let cookies = store.jars.entry(identity).or_default();
            cookies.retain(|cookie| cookie.expires.is_none_or(|expires| expires > now));
            cookies.clone()
        })
        .unwrap_or_default()
}

pub(crate) fn set_from_cookie_store(
    scope: &mut v8::PinScope<'_, '_>,
    cookie: Cookie,
) -> Option<Cookie> {
    let document = super::document_global::value(scope)?;
    let identity = document.get_identity_hash().get();
    let now = now_seconds(scope);
    let store = scope.get_slot_mut::<DocumentCookieStore>()?;
    let cookies = store.jars.entry(identity).or_default();
    cookies.retain(|cookie| cookie.expires.is_none_or(|expires| expires > now));
    let old = cookies
        .iter()
        .find(|existing| {
            existing.name == cookie.name
                && existing.domain == cookie.domain
                && existing.path == cookie.path
        })
        .cloned();
    cookies.retain(|existing| {
        existing.name != cookie.name
            || existing.domain != cookie.domain
            || existing.path != cookie.path
    });
    cookies.push(cookie);
    old
}

pub(crate) fn delete_from_cookie_store(
    scope: &mut v8::PinScope<'_, '_>,
    name: &str,
    domain: Option<&str>,
    path: Option<&str>,
) -> Option<Cookie> {
    let document = super::document_global::value(scope)?;
    let identity = document.get_identity_hash().get();
    let store = scope.get_slot_mut::<DocumentCookieStore>()?;
    let cookies = store.jars.entry(identity).or_default();
    let position = cookies.iter().position(|cookie| {
        cookie.name == name
            && domain.is_none_or(|domain| cookie.domain == domain)
            && path.is_none_or(|path| cookie.path == path)
    })?;
    Some(cookies.remove(position))
}

fn valid_cookie_name_byte(byte: u8) -> bool {
    byte > 0x20
        && byte < 0x7f
        && !matches!(
            byte,
            b'(' | b')'
                | b'<'
                | b'>'
                | b'@'
                | b','
                | b';'
                | b':'
                | b'\\'
                | b'"'
                | b'/'
                | b'['
                | b']'
                | b'?'
                | b'='
                | b'{'
                | b'}'
        )
}

fn valid_cookie_value_byte(byte: u8) -> bool {
    byte >= 0x20 && byte < 0x7f && !matches!(byte, b';' | b'\r' | b'\n' | 0)
}

fn domain_matches(host: &str, domain: &str, host_only: bool) -> bool {
    host == domain || (!host_only && host.ends_with(&format!(".{domain}")))
}

fn path_matches(request_path: &str, cookie_path: &str) -> bool {
    if request_path == cookie_path {
        return true;
    }
    request_path.starts_with(cookie_path)
        && (cookie_path.ends_with('/')
            || request_path.as_bytes().get(cookie_path.len()) == Some(&b'/'))
}

fn default_cookie_path(request_path: &str) -> String {
    if !request_path.starts_with('/') || request_path.matches('/').count() <= 1 {
        return "/".to_owned();
    }
    request_path
        .rfind('/')
        .map_or_else(|| "/".to_owned(), |index| request_path[..index].to_owned())
}

fn parse_cookie_date(input: &str) -> Option<i64> {
    let normalized = input.replace([',', '-'], " ");
    let tokens = normalized.split_ascii_whitespace().collect::<Vec<_>>();
    let month_position = tokens
        .iter()
        .position(|token| month_number(token).is_some())?;
    let day = tokens
        .get(month_position.checked_sub(1)?)?
        .parse::<u32>()
        .ok()?;
    let month = month_number(tokens[month_position])?;
    let mut year = tokens.get(month_position + 1)?.parse::<i32>().ok()?;
    if (0..=69).contains(&year) {
        year += 2000;
    } else if (70..=99).contains(&year) {
        year += 1900;
    }
    if year < 1601 || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    let time = tokens
        .iter()
        .find(|token| token.bytes().filter(|byte| *byte == b':').count() == 2)?;
    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse::<i64>().ok()?;
    let minute = time_parts.next()?.parse::<i64>().ok()?;
    let second = time_parts.next()?.parse::<i64>().ok()?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(
        days_from_civil(year, month, day)
            .saturating_mul(86_400)
            .saturating_add(hour * 3_600 + minute * 60 + second),
    )
}

fn month_number(value: &str) -> Option<u32> {
    match value.to_ascii_lowercase().as_str() {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let adjusted_year = year - i32::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month as i32 + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day as i32 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    (era * 146_097 + day_of_era - 719_468) as i64
}
