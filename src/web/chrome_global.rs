pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let chrome = v8::Object::new(scope);
    let load_times =
        crate::webidl::create_function(scope, "", 0, v8::ConstructorBehavior::Throw, load_times)?;
    define(scope, chrome, "loadTimes", load_times.into())?;
    let csi = crate::webidl::create_function(scope, "", 0, v8::ConstructorBehavior::Throw, csi)?;
    define(scope, chrome, "csi", csi.into())?;
    let app = create_app(scope)?;
    define(scope, chrome, "app", app.into())?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "chrome")?;
    if global.define_own_property(
        scope,
        key.into(),
        chrome.into(),
        v8::PropertyAttribute::DONT_DELETE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.chrome".to_owned())
    }
}

fn create_app<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Object>, String> {
    let app = v8::Object::new(scope);
    define(
        scope,
        app,
        "isInstalled",
        v8::Boolean::new(scope, false).into(),
    )?;
    let get_details = crate::webidl::create_function(
        scope,
        "getDetails",
        0,
        v8::ConstructorBehavior::Throw,
        get_details,
    )?;
    define(scope, app, "getDetails", get_details.into())?;
    let get_is_installed = crate::webidl::create_function(
        scope,
        "getIsInstalled",
        0,
        v8::ConstructorBehavior::Throw,
        get_is_installed,
    )?;
    define(scope, app, "getIsInstalled", get_is_installed.into())?;
    let install_state = crate::webidl::create_function(
        scope,
        "installState",
        0,
        v8::ConstructorBehavior::Throw,
        install_state,
    )?;
    define(scope, app, "installState", install_state.into())?;
    let running_state = crate::webidl::create_function(
        scope,
        "runningState",
        0,
        v8::ConstructorBehavior::Throw,
        running_state,
    )?;
    define(scope, app, "runningState", running_state.into())?;
    let install_values = v8::Object::new(scope);
    define_string(scope, install_values, "DISABLED", "disabled")?;
    define_string(scope, install_values, "INSTALLED", "installed")?;
    define_string(scope, install_values, "NOT_INSTALLED", "not_installed")?;
    define(scope, app, "InstallState", install_values.into())?;
    let running_values = v8::Object::new(scope);
    define_string(scope, running_values, "CANNOT_RUN", "cannot_run")?;
    define_string(scope, running_values, "READY_TO_RUN", "ready_to_run")?;
    define_string(scope, running_values, "RUNNING", "running")?;
    define(scope, app, "RunningState", running_values.into())?;
    Ok(app)
}

fn load_times(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let now = epoch_seconds(scope);
    let value = v8::Object::new(scope);
    let _ = define_number(scope, value, "requestTime", now);
    let _ = define_number(scope, value, "startLoadTime", now);
    let _ = define_number(scope, value, "commitLoadTime", 0.0);
    let _ = define_number(scope, value, "finishDocumentLoadTime", now);
    let _ = define_number(scope, value, "finishLoadTime", now);
    let _ = define_number(scope, value, "firstPaintTime", 0.0);
    let _ = define_number(scope, value, "firstPaintAfterLoadTime", 0.0);
    let _ = define_string(scope, value, "navigationType", "Other");
    let _ = define_boolean(scope, value, "wasFetchedViaSpdy", false);
    let _ = define_boolean(scope, value, "wasNpnNegotiated", false);
    let _ = define_string(scope, value, "npnNegotiatedProtocol", "");
    let _ = define_boolean(scope, value, "wasAlternateProtocolAvailable", false);
    let _ = define_string(scope, value, "connectionInfo", "unknown");
    result.set(value.into());
}

fn csi(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let now = epoch_seconds(scope);
    let value = v8::Object::new(scope);
    let _ = define_number(scope, value, "startE", (now * 1000.0).floor());
    let _ = define_number(scope, value, "onloadT", (now * 1000.0).floor());
    let _ = define_number(scope, value, "pageT", 0.0);
    let _ = define_number(scope, value, "tran", 15.0);
    result.set(value.into());
}

fn get_details(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::null(scope).into());
}

fn get_is_installed(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Boolean::new(scope, false).into());
}

fn install_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) {
        let state = v8::String::new(scope, "not_installed").expect("short install state");
        let receiver = v8::undefined(scope);
        let callback_arguments = [state.into()];
        let _ = callback.call(scope, receiver.into(), &callback_arguments);
    }
    result.set(v8::undefined(scope).into());
}

fn running_state(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = v8::String::new(scope, "cannot_run") {
        result.set(value.into());
    }
}

fn epoch_seconds(scope: &v8::PinScope<'_, '_>) -> f64 {
    crate::determinism::epoch_milliseconds(scope) / 1_000.0
}

fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::NONE)
        == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define chrome.{name}"))
    }
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) -> Result<(), String> {
    define(
        scope,
        object,
        name,
        crate::webidl::string(scope, value)?.into(),
    )
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) -> Result<(), String> {
    define(scope, object, name, v8::Number::new(scope, value).into())
}

fn define_boolean(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: bool,
) -> Result<(), String> {
    define(scope, object, name, v8::Boolean::new(scope, value).into())
}
