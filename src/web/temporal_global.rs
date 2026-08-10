pub(crate) fn install(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "Temporal")?;
    if global.define_own_property(scope, key.into(), value, v8::PropertyAttribute::DONT_ENUM)
        != Some(true)
    {
        return Err("cannot define window.Temporal".to_owned());
    }
    if crate::determinism::clock_is_deterministic(scope) {
        install_deterministic_now(scope, value)?;
    }
    Ok(())
}

fn install_deterministic_now(
    scope: &mut v8::PinScope<'_, '_>,
    temporal: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let temporal = v8::Local::<v8::Object>::try_from(temporal)
        .map_err(|_| "Temporal intrinsic is not an object".to_owned())?;
    let now_key = crate::webidl::string(scope, "Now")?;
    let now = temporal
        .get(scope, now_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "Temporal.Now is unavailable".to_owned())?;
    install_now_method(scope, now, "instant", temporal_now_instant)?;
    install_now_method(scope, now, "timeZoneId", temporal_now_time_zone_id)?;
    install_now_method(
        scope,
        now,
        "plainDateTimeISO",
        temporal_now_plain_date_time_iso,
    )?;
    install_now_method(
        scope,
        now,
        "zonedDateTimeISO",
        temporal_now_zoned_date_time_iso,
    )?;
    install_now_method(scope, now, "plainDateISO", temporal_now_plain_date_iso)?;
    install_now_method(scope, now, "plainTimeISO", temporal_now_plain_time_iso)
}

fn install_now_method(
    scope: &mut v8::PinScope<'_, '_>,
    now: v8::Local<'_, v8::Object>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, name, 0, v8::ConstructorBehavior::Throw, callback)?;
    let key = crate::webidl::string(scope, name)?;
    if now.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot install Temporal.Now.{name}"))
    }
}

fn temporal_now_instant(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(instant) = instant_from_clock(scope) {
        result.set(instant.into());
    }
}

fn temporal_now_time_zone_id(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let time_zone = crate::fingerprint::edge(scope).locale.time_zone.clone();
    if let Some(time_zone) = v8::String::new(scope, &time_zone) {
        result.set(time_zone.into());
    }
}

fn temporal_now_plain_date_time_iso(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_zoned_or_plain(scope, arguments, result, Some("toPlainDateTime"));
}

fn temporal_now_zoned_date_time_iso(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_zoned_or_plain(scope, arguments, result, None);
}

fn temporal_now_plain_date_iso(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_zoned_or_plain(scope, arguments, result, Some("toPlainDate"));
}

fn temporal_now_plain_time_iso(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_zoned_or_plain(scope, arguments, result, Some("toPlainTime"));
}

fn return_zoned_or_plain(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    conversion: Option<&str>,
) {
    let Some(instant) = instant_from_clock(scope) else {
        return;
    };
    let time_zone = if arguments.length() > 0 && !arguments.get(0).is_undefined() {
        v8::Global::new(scope, arguments.get(0))
    } else {
        let configured = crate::fingerprint::edge(scope).locale.time_zone.clone();
        let Some(configured) = v8::String::new(scope, &configured) else {
            return;
        };
        let configured: v8::Local<v8::Value> = configured.into();
        v8::Global::new(scope, configured)
    };
    let time_zone = v8::Local::new(scope, &time_zone);
    let Some(zoned) = call_method(scope, instant, "toZonedDateTimeISO", &[time_zone]) else {
        return;
    };
    if let Some(conversion) = conversion {
        let Ok(zoned) = v8::Local::<v8::Object>::try_from(zoned) else {
            return;
        };
        if let Some(plain) = call_method(scope, zoned, conversion, &[]) {
            result.set(plain);
        }
    } else {
        result.set(zoned);
    }
}

fn instant_from_clock<'s>(scope: &mut v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::Object>> {
    let global = scope.get_current_context().global(scope);
    let temporal_key = v8::String::new(scope, "Temporal")?;
    let temporal = global.get(scope, temporal_key.into())?;
    let temporal = v8::Local::<v8::Object>::try_from(temporal).ok()?;
    let instant_key = v8::String::new(scope, "Instant")?;
    let instant_constructor = temporal.get(scope, instant_key.into())?;
    let instant_constructor = v8::Local::<v8::Object>::try_from(instant_constructor).ok()?;
    let from_key = v8::String::new(scope, "fromEpochNanoseconds")?;
    let from = instant_constructor.get(scope, from_key.into())?;
    let from = v8::Local::<v8::Function>::try_from(from).ok()?;
    let epoch_ns = crate::determinism::epoch_nanoseconds(scope);
    let epoch_ns = i64::try_from(epoch_ns).ok()?;
    let epoch_ns = v8::BigInt::new_from_i64(scope, epoch_ns);
    let instant = from.call(scope, instant_constructor.into(), &[epoch_ns.into()])?;
    v8::Local::<v8::Object>::try_from(instant).ok()
}

fn call_method<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    receiver: v8::Local<'s, v8::Object>,
    name: &str,
    arguments: &[v8::Local<'s, v8::Value>],
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let method = receiver.get(scope, key.into())?;
    let method = v8::Local::<v8::Function>::try_from(method).ok()?;
    method.call(scope, receiver.into(), arguments)
}
