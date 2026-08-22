const MAX_HOOKS: usize = 64;
const MAX_HOOK_NAME_BYTES: usize = 128;
const MAX_HOOK_SOURCE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_SOURCE_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const ROOT_PRELOAD_HOOK_NAME: &str = "__edge_root_preload__";

/// JavaScript executed inside every iframe realm before that frame's page
/// scripts run.
///
/// The source runs in the iframe itself with a private `__edgev8` parameter.
/// It may deliberately replace realm-local functions such as
/// `XMLHttpRequest.prototype.send`. `__edgev8` is not a Window property and
/// the sandbox does not construct a JavaScript Proxy.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct IframeHook {
    pub name: String,
    pub source: String,
}

impl IframeHook {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() || self.name.len() > MAX_HOOK_NAME_BYTES {
            return Err("iframe hook name must contain 1 to 128 UTF-8 bytes".to_owned());
        }
        if self
            .name
            .chars()
            .any(|character| character.is_control() || matches!(character, '/' | '\\'))
        {
            return Err("iframe hook name contains an invalid character".to_owned());
        }
        if self.source.is_empty() {
            return Err(format!("iframe hook '{}' has empty source", self.name));
        }
        if self.source.len() > MAX_HOOK_SOURCE_BYTES {
            return Err(format!(
                "iframe hook '{}' exceeds 1048576 source bytes",
                self.name
            ));
        }
        Ok(())
    }
}

#[derive(Default)]
struct IframeHookStore {
    hooks: Vec<IframeHook>,
    protection_realms: Vec<std::pin::Pin<Box<ProtectionRealm>>>,
}

struct ProtectedFunction {
    function: v8::Global<v8::Function>,
    native_name: String,
}

struct ProtectionRealm {
    original_to_string: v8::Global<v8::Function>,
    protected: std::cell::RefCell<Vec<ProtectedFunction>>,
}

pub(crate) fn validate_hooks(hooks: &[IframeHook]) -> Result<(), String> {
    if hooks.len() > MAX_HOOKS {
        return Err("iframe hook configuration contains more than 64 hooks".to_owned());
    }
    let mut total = 0usize;
    for hook in hooks {
        hook.validate()?;
        total = total.saturating_add(hook.source.len());
    }
    if total > MAX_TOTAL_SOURCE_BYTES {
        return Err("iframe hook sources exceed 4194304 bytes in total".to_owned());
    }
    Ok(())
}

pub(crate) fn prepare(
    isolate: &mut v8::OwnedIsolate,
    hooks: Vec<IframeHook>,
) -> Result<(), String> {
    validate_hooks(&hooks)?;
    isolate.set_slot(IframeHookStore {
        hooks,
        protection_realms: Vec::new(),
    });
    Ok(())
}

pub(crate) fn run_for_current_iframe(
    scope: &mut v8::PinScope<'_, '_>,
    frame_url: &str,
) -> Result<(), String> {
    let hooks = scope
        .get_slot::<IframeHookStore>()
        .map(|store| store.hooks.clone())
        .unwrap_or_default();
    if hooks.is_empty() {
        return Ok(());
    }
    let private_api = install_native_source_protection(scope)?;
    for hook in hooks
        .into_iter()
        .filter(|hook| hook.name != ROOT_PRELOAD_HOOK_NAME)
    {
        run_one(scope, frame_url, &hook, private_api)?;
    }
    Ok(())
}

const PROXY_METHOD: &str = "proxy";
const PROTECTION_METHOD: &str = "protectPrototypeFunction";
const MAX_PROTECTED_FUNCTIONS_PER_REALM: usize = 4096;

fn install_native_source_protection<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let data = install_to_string_protection(scope)?;
    let proxy = crate::webidl::create_function_with_data(
        scope,
        PROXY_METHOD,
        1,
        v8::ConstructorBehavior::Throw,
        proxy_native_function,
        data.into(),
    )?;
    let protector = crate::webidl::create_function_with_data(
        scope,
        PROTECTION_METHOD,
        2,
        v8::ConstructorBehavior::Throw,
        protect_native_function,
        data.into(),
    )?;
    let v8_object = v8::Object::new(scope);
    define_private_method(scope, v8_object, PROXY_METHOD, proxy)?;
    define_private_method(scope, v8_object, PROTECTION_METHOD, protector)?;
    Ok(v8_object)
}

pub(crate) fn install_for_root(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let hooks = scope
        .get_slot::<IframeHookStore>()
        .map(|store| store.hooks.clone())
        .unwrap_or_default();
    if hooks.is_empty() {
        return Ok(());
    }
    let private_api = install_native_source_protection(scope)?;
    let frame_url = crate::page_init::url(scope);
    for hook in hooks
        .iter()
        .filter(|hook| hook.name == ROOT_PRELOAD_HOOK_NAME)
    {
        run_one(scope, &frame_url, hook, private_api)?;
    }
    Ok(())
}

fn install_to_string_protection<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::External>, String> {
    let global = scope.get_current_context().global(scope);
    let function_key = crate::webidl::string(scope, "Function")?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    let to_string_key = crate::webidl::string(scope, "toString")?;
    let function_constructor = global
        .get(scope, function_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "iframe Function constructor is unavailable".to_owned())?;
    let function_prototype = function_constructor
        .get(scope, prototype_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "iframe Function.prototype is unavailable".to_owned())?;
    let original_to_string = function_prototype
        .get(scope, to_string_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "iframe Function.prototype.toString is unavailable".to_owned())?;

    let record = Box::pin(ProtectionRealm {
        original_to_string: v8::Global::new(scope, original_to_string),
        protected: std::cell::RefCell::new(Vec::new()),
    });
    let pointer = (&*record as *const ProtectionRealm).cast_mut();
    scope
        .get_slot_mut::<IframeHookStore>()
        .ok_or_else(|| "iframe hook state was not prepared".to_owned())?
        .protection_realms
        .push(record);

    let data = v8::External::new(scope, pointer.cast());
    let protected_to_string = crate::webidl::create_function_with_data(
        scope,
        "toString",
        0,
        v8::ConstructorBehavior::Throw,
        protected_function_to_string,
        data.into(),
    )?;
    let mut to_string_descriptor =
        v8::PropertyDescriptor::new_from_value_writable(protected_to_string.into(), true);
    to_string_descriptor.set_enumerable(false);
    to_string_descriptor.set_configurable(true);
    if function_prototype.define_property(scope, to_string_key.into(), &to_string_descriptor)
        != Some(true)
    {
        return Err("cannot install protected Function.prototype.toString".to_owned());
    }

    Ok(data)
}

fn define_private_method(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    function: v8::Local<'_, v8::Function>,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let mut descriptor = v8::PropertyDescriptor::new_from_value_writable(function.into(), false);
    descriptor.set_enumerable(false);
    descriptor.set_configurable(false);
    if object.define_property(scope, key.into(), &descriptor) != Some(true) {
        return Err(format!("cannot define private V8 hook method {name}"));
    }
    Ok(())
}

fn proxy_native_function(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut return_value: v8::ReturnValue<'_>,
) {
    let Ok(function) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "First argument must be a function");
        return;
    };
    let Some(record) = protection_realm(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Native-source protection is unavailable");
        return;
    };
    let native_name = if arguments.get(1).is_undefined() {
        function.get_name(scope).to_rust_string_lossy(scope)
    } else {
        let Some(name) = arguments.get(1).to_string(scope) else {
            return;
        };
        name.to_rust_string_lossy(scope)
    };
    register_protected_function(scope, record, function, native_name, &mut return_value);
}

fn register_protected_function(
    scope: &mut v8::PinScope<'_, '_>,
    record: &ProtectionRealm,
    function: v8::Local<'_, v8::Function>,
    native_name: String,
    return_value: &mut v8::ReturnValue<'_>,
) {
    if native_name.len() > MAX_HOOK_NAME_BYTES || native_name.contains(['\r', '\n', '\0']) {
        crate::webidl::throw_type_error(scope, "Native function name is invalid");
        return;
    }
    let mut protected = record.protected.borrow_mut();
    let hash = function.get_identity_hash().get();
    if let Some(existing) = protected.iter_mut().find(|entry| {
        v8::Local::new(scope, &entry.function)
            .get_identity_hash()
            .get()
            == hash
            && v8::Local::new(scope, &entry.function).strict_equals(function.into())
    }) {
        existing.native_name = native_name;
        return_value.set(function.into());
        return;
    }
    if protected.len() >= MAX_PROTECTED_FUNCTIONS_PER_REALM {
        crate::webidl::throw_type_error(scope, "Too many protected functions in this iframe realm");
        return;
    }
    protected.push(ProtectedFunction {
        function: v8::Global::new(scope, function),
        native_name,
    });
    return_value.set(function.into());
}

fn protection_realm<'a>(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<&'a ProtectionRealm> {
    let data = crate::trace::native_callback_data(scope, arguments);
    let external = v8::Local::<v8::External>::try_from(data).ok()?;
    let pointer = external.value().cast::<ProtectionRealm>();
    if pointer.is_null() {
        None
    } else {
        // SAFETY: records are pinned and retained in the isolate slot for the
        // lifetime of every native function carrying this pointer.
        Some(unsafe { &*pointer })
    }
}

fn protect_native_function(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut return_value: v8::ReturnValue<'_>,
) {
    let Ok(prototype) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "First argument must be a prototype object");
        return;
    };
    let Some(record) = protection_realm(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Native-source protection is unavailable");
        return;
    };
    let Some(property_name) = arguments.get(1).to_string(scope) else {
        return;
    };
    let native_name = property_name.to_rust_string_lossy(scope);
    let Some(descriptor) = prototype.get_own_property_descriptor(scope, property_name.into())
    else {
        crate::webidl::throw_type_error(scope, "Prototype property does not exist");
        return;
    };
    let Ok(descriptor) = v8::Local::<v8::Object>::try_from(descriptor) else {
        crate::webidl::throw_type_error(scope, "Prototype descriptor is invalid");
        return;
    };
    let Some(value_key) = v8::String::new(scope, "value") else {
        return;
    };
    let Some(value) = descriptor.get(scope, value_key.into()) else {
        return;
    };
    let Ok(function) = v8::Local::<v8::Function>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "Prototype property value must be a function");
        return;
    };
    register_protected_function(scope, record, function, native_name, &mut return_value);
}

fn protected_function_to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut return_value: v8::ReturnValue<'_>,
) {
    let Some(record) = protection_realm(scope, &arguments) else {
        crate::webidl::throw_type_error(scope, "Native-source protection is unavailable");
        return;
    };
    let receiver: v8::Local<'_, v8::Value> = arguments.this().into();
    if let Ok(function) = v8::Local::<v8::Function>::try_from(receiver) {
        if let Some(native_name) = protected_native_name(scope, function) {
            let text = format!("function {native_name}() {{ [native code] }}");
            if let Some(text) = v8::String::new(scope, &text) {
                return_value.set(text.into());
            }
            return;
        }
    }
    let original = v8::Local::new(scope, &record.original_to_string);
    if let Some(value) = original.call(scope, receiver, &[]) {
        return_value.set(value);
    }
}

fn protected_native_name(
    scope: &v8::PinScope<'_, '_>,
    function: v8::Local<'_, v8::Function>,
) -> Option<String> {
    let hash = function.get_identity_hash().get();
    let store = scope.get_slot::<IframeHookStore>()?;
    for realm in &store.protection_realms {
        let protected = realm.protected.borrow();
        if let Some(entry) = protected.iter().find(|entry| {
            v8::Local::new(scope, &entry.function)
                .get_identity_hash()
                .get()
                == hash
                && v8::Local::new(scope, &entry.function).strict_equals(function.into())
        }) {
            return Some(entry.native_name.clone());
        }
    }
    None
}

fn run_one(
    scope: &mut v8::PinScope<'_, '_>,
    frame_url: &str,
    hook: &IframeHook,
    private_api: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    v8::tc_scope!(let try_catch, scope);
    let wrapped = format!("(function(__edgev8) {{\n{}\n}})", hook.source);
    let source = v8::String::new(try_catch, &wrapped)
        .ok_or_else(|| format!("iframe hook '{}' exceeds V8 limits", hook.name))?;
    let resource = format!("edge-sandbox-iframe-hook://{}", hook.name);
    let resource_name = v8::String::new(try_catch, &resource)
        .ok_or_else(|| format!("iframe hook '{}' resource name is invalid", hook.name))?;
    let origin = v8::ScriptOrigin::new(
        try_catch,
        resource_name.into(),
        0,
        0,
        false,
        -1,
        None,
        false,
        false,
        false,
        None,
    );
    let result = v8::Script::compile(try_catch, source, Some(&origin))
        .and_then(|script| script.run(try_catch))
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .and_then(|function| {
            let receiver: v8::Local<'_, v8::Value> =
                try_catch.get_current_context().global(try_catch).into();
            function.call(try_catch, receiver, &[private_api.into()])
        });
    if result.is_some() {
        try_catch.perform_microtask_checkpoint();
        return Ok(());
    }
    let detail = try_catch
        .exception()
        .and_then(|exception| exception.to_string(try_catch))
        .map(|text| text.to_rust_string_lossy(try_catch))
        .unwrap_or_else(|| "compilation or execution failed".to_owned());
    Err(format!(
        "iframe hook '{}' failed for {}: {}",
        hook.name, frame_url, detail
    ))
}
