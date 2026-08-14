use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WorkerNavigatorStore {
    languages: HashMap<i32, v8::Global<v8::Array>>,
    services: HashMap<i32, WorkerNavigatorServices>,
}

#[derive(Clone)]
struct WorkerNavigatorServices {
    connection: v8::Global<v8::Object>,
    gpu: v8::Global<v8::Object>,
    hid: v8::Global<v8::Object>,
    locks: v8::Global<v8::Object>,
    media_capabilities: v8::Global<v8::Object>,
    permissions: v8::Global<v8::Object>,
    serial: Option<v8::Global<v8::Object>>,
    storage_buckets: v8::Global<v8::Object>,
    storage: v8::Global<v8::Object>,
    usb: v8::Global<v8::Object>,
    user_agent_data: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WorkerNavigatorStore::default());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<(v8::Local<'s, v8::Function>, v8::Local<'s, v8::Object>), String> {
    let constructor = crate::webidl::create_function(
        scope,
        "WorkerNavigator",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::worker_navigator_hardware_concurrency_property::define(scope, prototype)?;
    super::worker_navigator_app_code_name_property::define(scope, prototype)?;
    super::worker_navigator_app_name_property::define(scope, prototype)?;
    super::worker_navigator_app_version_property::define(scope, prototype)?;
    super::worker_navigator_platform_property::define(scope, prototype)?;
    super::worker_navigator_product_property::define(scope, prototype)?;
    super::worker_navigator_user_agent_property::define(scope, prototype)?;
    super::worker_navigator_language_property::define(scope, prototype)?;
    super::worker_navigator_languages_property::define(scope, prototype)?;
    super::worker_navigator_on_line_property::define(scope, prototype)?;
    super::worker_navigator_connection_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::worker_navigator_hid_property::define(scope, prototype)?;
    super::worker_navigator_media_capabilities_property::define(scope, prototype)?;
    super::worker_navigator_permissions_property::define(scope, prototype)?;
    if super::worker_global_scope::current_record(scope)
        .is_some_and(|record| record.kind == super::worker_global_scope::RealmKind::Dedicated)
    {
        super::worker_navigator_serial_property::define(scope, prototype)?;
    }
    super::worker_navigator_usb_property::define(scope, prototype)?;
    super::worker_navigator_device_memory_property::define(scope, prototype)?;
    super::worker_navigator_user_agent_data_property::define(scope, prototype)?;
    super::worker_navigator_locks_property::define(scope, prototype)?;
    super::worker_navigator_storage_property::define(scope, prototype)?;
    super::worker_navigator_gpu_property::define(scope, prototype)?;
    super::worker_navigator_storage_buckets_property::define(scope, prototype)?;
    if super::worker_global_scope::current_record(scope)
        .is_some_and(|record| record.kind == super::worker_global_scope::RealmKind::Dedicated)
    {
        let version = crate::browser_surface::current_version(scope);
        crate::browser_surface::reorder_string_properties(
            scope,
            prototype,
            crate::browser_surface::worker_navigator_names(version),
            "WorkerNavigator.prototype",
        )?;
    }
    let navigator = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, navigator, prototype.into()) != Some(true) {
        return Err("cannot create WorkerNavigator".to_owned());
    }
    let configured = crate::fingerprint::navigator(scope).languages.clone();
    let languages = v8::Array::new(scope, configured.len() as i32);
    for (index, language) in configured.iter().enumerate() {
        set_language(scope, languages, index as u32, language);
    }
    let _ = languages.set_integrity_level(scope, v8::IntegrityLevel::Frozen);
    let connection = super::network_information::create(scope)?;
    let gpu = super::gpu::create(scope)?;
    let hid = super::hid::create(scope)?;
    let locks = super::lock_manager::create(scope)?;
    let media_capabilities = super::media_capabilities::create(scope)?;
    let permissions = super::permissions::create(scope)?;
    let serial = if super::worker_global_scope::current_record(scope)
        .is_some_and(|record| record.kind == super::worker_global_scope::RealmKind::Dedicated)
    {
        let serial = super::serial::create(scope)?;
        Some(v8::Global::new(scope, serial))
    } else {
        None
    };
    let storage_buckets = super::storage_bucket_manager::create(scope)?;
    let storage = super::storage_manager::create(scope)?;
    let usb = super::usb::create(scope)?;
    let user_agent_data = super::navigator_ua_data::create(scope)?;
    let identity = navigator.get_identity_hash().get();
    let languages = v8::Global::new(scope, languages);
    let services = WorkerNavigatorServices {
        connection: v8::Global::new(scope, connection),
        gpu: v8::Global::new(scope, gpu),
        hid: v8::Global::new(scope, hid),
        locks: v8::Global::new(scope, locks),
        media_capabilities: v8::Global::new(scope, media_capabilities),
        permissions: v8::Global::new(scope, permissions),
        serial,
        storage_buckets: v8::Global::new(scope, storage_buckets),
        storage: v8::Global::new(scope, storage),
        usb: v8::Global::new(scope, usb),
        user_agent_data: v8::Global::new(scope, user_agent_data),
    };
    let store = scope
        .get_slot_mut::<WorkerNavigatorStore>()
        .ok_or_else(|| "WorkerNavigator state was not prepared".to_owned())?;
    store.languages.insert(identity, languages);
    store.services.insert(identity, services);
    Ok((constructor, navigator))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn valid_this(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    let valid = super::worker_global_scope::current_record(scope)
        .and_then(|record| record.navigator)
        .is_some_and(|navigator| {
            v8::Local::new(scope, &navigator).get_identity_hash().get()
                == object.get_identity_hash().get()
        });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    valid
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

pub(crate) fn set_language(
    scope: &v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    index: u32,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        let _ = array.set_index(scope, index, value.into());
    }
}

pub(crate) fn languages_object(
    scope: &v8::PinScope<'_, '_>,
    navigator: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Array>> {
    scope
        .get_slot::<WorkerNavigatorStore>()?
        .languages
        .get(&navigator.get_identity_hash().get())
        .cloned()
}

fn return_service(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&WorkerNavigatorServices) -> Option<v8::Global<v8::Object>>,
) {
    if !valid_this(scope, arguments.this()) {
        return;
    }
    let value = scope
        .get_slot::<WorkerNavigatorStore>()
        .and_then(|store| {
            store
                .services
                .get(&arguments.this().get_identity_hash().get())
        })
        .and_then(select);
    match value {
        Some(value) => result.set(v8::Local::new(scope, &value).into()),
        None => result.set(v8::undefined(scope).into()),
    }
}

pub(crate) fn get_connection(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.connection.clone())
    });
}

pub(crate) fn get_gpu(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.gpu.clone())
    });
}

pub(crate) fn get_hid(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.hid.clone())
    });
}

pub(crate) fn get_locks(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.locks.clone())
    });
}

pub(crate) fn get_media_capabilities(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.media_capabilities.clone())
    });
}

pub(crate) fn get_permissions(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.permissions.clone())
    });
}

pub(crate) fn get_serial(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| services.serial.clone());
}

pub(crate) fn get_storage_buckets(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.storage_buckets.clone())
    });
}

pub(crate) fn get_storage(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.storage.clone())
    });
}

pub(crate) fn get_usb(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.usb.clone())
    });
}

pub(crate) fn get_user_agent_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_service(scope, arguments, result, |services| {
        Some(services.user_agent_data.clone())
    });
}
