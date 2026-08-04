unsafe extern "C" {
    #[link_name = "ucal_setDefaultTimeZone_77"]
    fn ucal_set_default_time_zone(zone_id: *const u16, error_code: *mut i32);
}

static PROCESS_DEFAULTS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn lock_process_defaults() -> std::sync::MutexGuard<'static, ()> {
    PROCESS_DEFAULTS_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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

pub(crate) fn prepare(_: &mut v8::OwnedIsolate) {}

/// Date and Intl now use their original V8 implementations. The configured
/// ICU defaults were installed before isolate creation, so every realm sees
/// the same real locale/time-zone behavior without JavaScript-visible wrappers.
pub(crate) fn install(_: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    Ok(())
}
