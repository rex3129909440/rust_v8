use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CrashReportContextStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CrashReportRecord>,
}

#[derive(Clone, Default)]
struct CrashReportRecord {
    initialized: bool,
    product_name: String,
    company_name: String,
    submit_url: String,
    annotations: HashMap<String, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CrashReportContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CrashReportContext", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CrashReportContextStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CrashReportContext",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "initialize", 1, initialize)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CrashReportContextStore>()
        .ok_or_else(|| "CrashReportContext state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CrashReportContext".to_owned());
    }
    scope
        .get_slot_mut::<CrashReportContextStore>()
        .ok_or_else(|| "CrashReportContext state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CrashReportRecord::default(),
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CrashReportContext': Illegal constructor",
    );
}

fn initialize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'initialize' on 'CrashReportContext': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "CrashReportContext options must be an object");
        return;
    };
    let product_name = string_property(scope, options, "productName");
    let company_name = string_property(scope, options, "companyName");
    let submit_url = string_property(scope, options, "submitURL");
    if let Some(record) = scope
        .get_slot_mut::<CrashReportContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.initialized = true;
        record.product_name = product_name;
        record.company_name = company_name;
        record.submit_url = submit_url;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(promise) =
        super::writable_stream::resolved_promise(scope, v8::undefined(scope).into())
    {
        result.set(promise.into());
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'set' on 'CrashReportContext': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    let Some(record) = scope
        .get_slot_mut::<CrashReportContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.initialized {
        throw_invalid_state(
            scope,
            "CrashReportContext is not initialized. Call initialize() and wait for it to resolve.",
        );
        return;
    }
    record.annotations.insert(key, value);
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'delete' on 'CrashReportContext': 1 argument required, but only 0 present.",
        );
        return;
    }
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<CrashReportContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.initialized {
        throw_invalid_state(
            scope,
            "CrashReportContext is not initialized. Call initialize() and wait for it to resolve.",
        );
        return;
    }
    record.annotations.remove(&key);
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> String {
    let Some(key) = v8::String::new(scope, name) else {
        return String::new();
    };
    object
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}

fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), "InvalidStateError".to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(_) => crate::webidl::throw_type_error(scope, message),
    }
}
