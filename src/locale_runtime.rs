unsafe extern "C" {
    #[link_name = "ucal_setDefaultTimeZone_77"]
    fn ucal_set_default_time_zone(zone_id: *const u16, error_code: *mut i32);
    #[link_name = "ucal_open_77"]
    fn ucal_open(
        zone_id: *const u16,
        zone_id_length: i32,
        locale: *const std::ffi::c_char,
        calendar_type: i32,
        error_code: *mut i32,
    ) -> *mut std::ffi::c_void;
    #[link_name = "ucal_close_77"]
    fn ucal_close(calendar: *mut std::ffi::c_void);
    #[link_name = "ucal_setMillis_77"]
    fn ucal_set_millis(calendar: *mut std::ffi::c_void, date_time: f64, error_code: *mut i32);
    #[link_name = "ucal_get_77"]
    fn ucal_get(calendar: *const std::ffi::c_void, field: i32, error_code: *mut i32) -> i32;
}

static PROCESS_DEFAULTS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Default)]
struct LocaleRuntimeState {
    process_time_zone: String,
    date_time_format_resolved_options: std::collections::HashMap<i32, v8::Global<v8::Function>>,
    webview_plural_rules_resolved_options: std::collections::HashMap<i32, v8::Global<v8::Function>>,
}

pub(crate) fn lock_process_defaults() -> std::sync::MutexGuard<'static, ()> {
    PROCESS_DEFAULTS_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn effective_process_time_zone(
    configured_time_zone: &str,
    configured_offset_minutes: i32,
) -> Result<String, String> {
    let actual = current_offset_minutes(configured_time_zone)?;
    if actual == configured_offset_minutes {
        Ok(configured_time_zone.to_owned())
    } else {
        Ok(fixed_offset_time_zone(configured_offset_minutes))
    }
}

fn current_offset_minutes(time_zone: &str) -> Result<i32, String> {
    const UCAL_DEFAULT: i32 = 0;
    const UCAL_ZONE_OFFSET: i32 = 15;
    const UCAL_DST_OFFSET: i32 = 16;
    let zone_id = time_zone.encode_utf16().collect::<Vec<_>>();
    let zone_id_length = i32::try_from(zone_id.len())
        .map_err(|_| "configured time zone identifier is too long".to_owned())?;
    let mut error_code = 0_i32;
    // SAFETY: ICU borrows the live UTF-16 slice for the duration of the call.
    let calendar = unsafe {
        ucal_open(
            zone_id.as_ptr(),
            zone_id_length,
            std::ptr::null(),
            UCAL_DEFAULT,
            &mut error_code,
        )
    };
    if calendar.is_null() || error_code > 0 {
        return Err(format!(
            "ICU rejected configured time zone '{time_zone}' with error {error_code}"
        ));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("system clock precedes Unix epoch: {error}"))?
        .as_secs_f64()
        * 1_000.0;
    // SAFETY: `calendar` remains live until the matching close below.
    unsafe { ucal_set_millis(calendar, now, &mut error_code) };
    let zone_offset = if error_code <= 0 {
        // SAFETY: the calendar remains live and both field constants are ICU ABI values.
        unsafe { ucal_get(calendar, UCAL_ZONE_OFFSET, &mut error_code) }
    } else {
        0
    };
    let dst_offset = if error_code <= 0 {
        // SAFETY: same live calendar and ABI contract as above.
        unsafe { ucal_get(calendar, UCAL_DST_OFFSET, &mut error_code) }
    } else {
        0
    };
    // SAFETY: the calendar was returned by `ucal_open` and is closed once.
    unsafe { ucal_close(calendar) };
    if error_code > 0 {
        return Err(format!(
            "ICU could not resolve configured time zone '{time_zone}' with error {error_code}"
        ));
    }
    Ok(-(zone_offset.saturating_add(dst_offset) / 60_000))
}

fn fixed_offset_time_zone(offset_minutes: i32) -> String {
    let utc_minutes = -offset_minutes;
    if utc_minutes == 0 {
        return "GMT".to_owned();
    }
    let sign = if utc_minutes < 0 { '-' } else { '+' };
    let absolute = utc_minutes.unsigned_abs();
    format!("GMT{sign}{:02}:{:02}", absolute / 60, absolute % 60)
}

/// Configures the process-wide ICU defaults before an isolate is created.
///
/// Production evaluations run in one isolated worker process per sandbox, so
/// ICU's process-wide defaults remain instance scoped from the caller's point
/// of view. Setting the real ICU time zone is required: changing only
/// `resolvedOptions()` or `Date#getTimezoneOffset()` leaves Date formatting,
/// Intl formatting, parsing, and daylight-saving transitions on the host zone.
pub(crate) fn configure_process_defaults(locale: &str, time_zone: &str) -> Result<(), String> {
    v8::icu::set_default_locale(locale);

    let mut zone_id = time_zone.encode_utf16().collect::<Vec<_>>();
    zone_id.push(0);
    let mut error_code = 0_i32;
    // SAFETY: `zone_id` is a live, NUL-terminated UTF-16 string for the
    // duration of the ICU call and `error_code` points to writable storage.
    unsafe {
        ucal_set_default_time_zone(zone_id.as_ptr(), &mut error_code);
    }
    if error_code == 0 {
        Ok(())
    } else {
        Err(format!(
            "ICU rejected configured time zone '{time_zone}' with error {error_code}"
        ))
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate, process_time_zone: String) {
    isolate.set_slot(LocaleRuntimeState {
        process_time_zone,
        ..LocaleRuntimeState::default()
    });
}

/// Date and Intl now use their original V8 implementations. The configured
/// ICU defaults were installed before isolate creation, so every realm sees
/// the same real locale/time-zone behavior without JavaScript-visible wrappers.
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    install_date_time_format_resolved_options(scope)?;
    let version = crate::browser_surface::current_version(scope);
    if !version.is_webview() || version.major() != 136 {
        return Ok(());
    }
    let global = scope.get_current_context().global(scope);
    let intl_key = crate::webidl::string(scope, "Intl")?;
    let intl = global
        .get(scope, intl_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "WebView Intl object is unavailable".to_owned())?;
    let plural_rules_key = crate::webidl::string(scope, "PluralRules")?;
    let plural_rules = intl
        .get(scope, plural_rules_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "WebView Intl.PluralRules is unavailable".to_owned())?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    let prototype = plural_rules
        .get(scope, prototype_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "WebView Intl.PluralRules.prototype is unavailable".to_owned())?;
    let resolved_options_key = crate::webidl::string(scope, "resolvedOptions")?;
    let original = prototype
        .get(scope, resolved_options_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "WebView Intl.PluralRules.resolvedOptions is unavailable".to_owned())?;
    let replacement = crate::webidl::create_function(
        scope,
        "resolvedOptions",
        0,
        v8::ConstructorBehavior::Throw,
        webview_plural_rules_resolved_options,
    )?;
    if prototype.set(scope, resolved_options_key.into(), replacement.into()) != Some(true) {
        return Err("cannot install WebView 136 PluralRules.resolvedOptions".to_owned());
    }
    let realm_id = crate::webidl::realm_id(scope);
    let original = v8::Global::new(scope, original);
    scope
        .get_slot_mut::<LocaleRuntimeState>()
        .ok_or_else(|| "locale runtime state was not prepared".to_owned())?
        .webview_plural_rules_resolved_options
        .insert(realm_id, original);
    Ok(())
}

fn install_date_time_format_resolved_options(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let intl_key = crate::webidl::string(scope, "Intl")?;
    let intl = global
        .get(scope, intl_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "Intl object is unavailable".to_owned())?;
    let constructor_key = crate::webidl::string(scope, "DateTimeFormat")?;
    let constructor = intl
        .get(scope, constructor_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "Intl.DateTimeFormat is unavailable".to_owned())?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    let prototype = constructor
        .get(scope, prototype_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "Intl.DateTimeFormat.prototype is unavailable".to_owned())?;
    let resolved_options_key = crate::webidl::string(scope, "resolvedOptions")?;
    let original = prototype
        .get(scope, resolved_options_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "Intl.DateTimeFormat.resolvedOptions is unavailable".to_owned())?;
    let replacement = crate::webidl::create_function(
        scope,
        "resolvedOptions",
        0,
        v8::ConstructorBehavior::Throw,
        date_time_format_resolved_options,
    )?;
    if prototype.set(scope, resolved_options_key.into(), replacement.into()) != Some(true) {
        return Err("cannot install Intl.DateTimeFormat.resolvedOptions".to_owned());
    }
    let realm_id = crate::webidl::realm_id(scope);
    let original = v8::Global::new(scope, original);
    scope
        .get_slot_mut::<LocaleRuntimeState>()
        .ok_or_else(|| "locale runtime state was not prepared".to_owned())?
        .date_time_format_resolved_options
        .insert(realm_id, original);
    Ok(())
}

fn date_time_format_resolved_options(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let realm_id = crate::webidl::realm_id(scope);
    let original = scope
        .get_slot::<LocaleRuntimeState>()
        .and_then(|state| state.date_time_format_resolved_options.get(&realm_id))
        .cloned();
    let Some(original) = original else {
        crate::webidl::throw_type_error(scope, "Intl.DateTimeFormat receiver is unavailable");
        return;
    };
    let original = v8::Local::new(scope, &original);
    let Some(options) = original.call(scope, arguments.this().into(), &[]) else {
        return;
    };
    let Ok(options) = v8::Local::<v8::Object>::try_from(options) else {
        result.set(options);
        return;
    };
    let Some(time_zone_key) = v8::String::new(scope, "timeZone") else {
        result.set(options.into());
        return;
    };
    let resolved = options
        .get(scope, time_zone_key.into())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let process_time_zone = scope
        .get_slot::<LocaleRuntimeState>()
        .map(|state| state.process_time_zone.clone())
        .unwrap_or_default();
    let resolved_is_process_zone = resolved.eq_ignore_ascii_case(&process_time_zone)
        || process_time_zone
            .strip_prefix("GMT")
            .is_some_and(|suffix| !suffix.is_empty() && resolved.eq_ignore_ascii_case(suffix))
        || (process_time_zone == "GMT" && resolved.eq_ignore_ascii_case("UTC"));
    if resolved_is_process_zone {
        let configured = crate::fingerprint::edge(scope).locale.time_zone.clone();
        if let Some(configured) = v8::String::new(scope, &configured) {
            let _ = options.set(scope, time_zone_key.into(), configured.into());
        }
    }
    result.set(options.into());
}

fn webview_plural_rules_resolved_options(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let realm_id = crate::webidl::realm_id(scope);
    let original = scope
        .get_slot::<LocaleRuntimeState>()
        .and_then(|state| state.webview_plural_rules_resolved_options.get(&realm_id))
        .cloned();
    let Some(original) = original else {
        crate::webidl::throw_type_error(scope, "Intl.PluralRules receiver is unavailable");
        return;
    };
    let original = v8::Local::new(scope, &original);
    let Some(options) = original.call(scope, arguments.this().into(), &[]) else {
        return;
    };
    let Ok(options) = v8::Local::<v8::Object>::try_from(options) else {
        result.set(options);
        return;
    };

    if let Some(notation) = v8::String::new(scope, "notation") {
        let _ = options.delete(scope, notation.into());
    }

    let locale_key = match v8::String::new(scope, "locale") {
        Some(value) => value,
        None => {
            result.set(options.into());
            return;
        }
    };
    let resolved_locale = options
        .get(scope, locale_key.into())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let configured_locale = crate::fingerprint::edge(scope).locale.locale.clone();
    if resolved_locale.eq_ignore_ascii_case(&configured_locale) {
        let navigator_locale = crate::fingerprint::navigator(scope).language.clone();
        if let Some(navigator_locale) = v8::String::new(scope, &navigator_locale) {
            let _ = options.set(scope, locale_key.into(), navigator_locale.into());
        }
    }
    result.set(options.into());
}
