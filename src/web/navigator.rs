use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigatorStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, NavigatorRecord>,
}

#[derive(Clone)]
struct NavigatorRecord {
    scheduling: v8::Global<v8::Object>,
    user_activation: v8::Global<v8::Object>,
    geolocation: v8::Global<v8::Object>,
    temporary_storage: v8::Global<v8::Object>,
    persistent_storage: v8::Global<v8::Object>,
    window_controls_overlay: v8::Global<v8::Object>,
    languages: v8::Global<v8::Array>,
    plugins: v8::Global<v8::Object>,
    mime_types: v8::Global<v8::Object>,
    connection: v8::Global<v8::Object>,
    protected_audience: v8::Global<v8::Object>,
    bluetooth: v8::Global<v8::Object>,
    clipboard: v8::Global<v8::Object>,
    credentials: v8::Global<v8::Object>,
    keyboard: v8::Global<v8::Object>,
    managed: v8::Global<v8::Object>,
    media_devices: v8::Global<v8::Object>,
    service_worker: v8::Global<v8::Object>,
    virtual_keyboard: v8::Global<v8::Object>,
    wake_lock: v8::Global<v8::Object>,
    user_agent_data: v8::Global<v8::Object>,
    locks: v8::Global<v8::Object>,
    storage: v8::Global<v8::Object>,
    gpu: v8::Global<v8::Object>,
    login: v8::Global<v8::Object>,
    ink: v8::Global<v8::Object>,
    media_capabilities: v8::Global<v8::Object>,
    permissions: v8::Global<v8::Object>,
    device_posture: v8::Global<v8::Object>,
    hid: v8::Global<v8::Object>,
    media_session: v8::Global<v8::Object>,
    presentation: v8::Global<v8::Object>,
    serial: v8::Global<v8::Object>,
    usb: v8::Global<v8::Object>,
    xr: v8::Global<v8::Object>,
    storage_buckets: v8::Global<v8::Object>,
    contacts: v8::Global<v8::Object>,
    model_context: v8::Global<v8::Object>,
    cookie_deprecation_label: v8::Global<v8::Object>,
    vibration_pattern: Vec<u32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigatorStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Navigator", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<NavigatorStore>()
        .and_then(|store| store.constructors.get(&realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let c = crate::webidl::create_function(
        scope,
        "Navigator",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::navigator_vendor_sub_property::define(scope, p)?;
    super::navigator_product_sub_property::define(scope, p)?;
    super::navigator_vendor_property::define(scope, p)?;
    super::navigator_max_touch_points_property::define(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "scheduling", get_scheduling)?;
    crate::webidl::define_readonly_accessor(scope, p, "userActivation", get_user_activation)?;
    crate::webidl::define_readonly_accessor(scope, p, "geolocation", get_geolocation)?;
    super::navigator_do_not_track_property::define(scope, p)?;
    crate::webidl::define_readonly_accessor(
        scope,
        p,
        "webkitTemporaryStorage",
        get_temporary_storage,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        p,
        "webkitPersistentStorage",
        get_persistent_storage,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        p,
        "windowControlsOverlay",
        get_window_controls_overlay,
    )?;
    super::navigator_hardware_concurrency_property::define(scope, p)?;
    super::navigator_cookie_enabled_property::define(scope, p)?;
    super::navigator_app_code_name_property::define(scope, p)?;
    super::navigator_app_name_property::define(scope, p)?;
    super::navigator_app_version_property::define(scope, p)?;
    super::navigator_platform_property::define(scope, p)?;
    super::navigator_product_property::define(scope, p)?;
    super::navigator_user_agent_property::define(scope, p)?;
    super::navigator_language_property::define(scope, p)?;
    super::navigator_languages_property::define(scope, p)?;
    super::navigator_on_line_property::define(scope, p)?;
    super::navigator_webdriver_property::define(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "plugins", get_plugins)?;
    crate::webidl::define_readonly_accessor(scope, p, "mimeTypes", get_mime_types)?;
    super::navigator_pdf_viewer_enabled_property::define(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "connection", get_connection)?;
    crate::webidl::define_method(scope, p, "getGamepads", 0, get_gamepads)?;
    crate::webidl::define_method(scope, p, "javaEnabled", 0, java_enabled)?;
    crate::webidl::define_method(scope, p, "sendBeacon", 1, send_beacon)?;
    crate::webidl::define_method(scope, p, "vibrate", 1, vibrate)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    super::navigator_model_context_property::define(scope, p)?;
    crate::webidl::define_readonly_accessor(
        scope,
        p,
        "deprecatedRunAdAuctionEnforcesKAnonymity",
        get_deprecated_run_ad_auction_enforces_k_anonymity,
    )?;
    crate::webidl::define_readonly_accessor(scope, p, "protectedAudience", get_protected_audience)?;
    crate::webidl::define_readonly_accessor(scope, p, "bluetooth", get_bluetooth)?;
    crate::webidl::define_readonly_accessor(scope, p, "clipboard", get_clipboard)?;
    crate::webidl::define_readonly_accessor(scope, p, "credentials", get_credentials)?;
    crate::webidl::define_readonly_accessor(scope, p, "keyboard", get_keyboard)?;
    crate::webidl::define_readonly_accessor(scope, p, "managed", get_managed)?;
    crate::webidl::define_readonly_accessor(scope, p, "mediaDevices", get_media_devices)?;
    crate::webidl::define_readonly_accessor(scope, p, "serviceWorker", get_service_worker)?;
    crate::webidl::define_readonly_accessor(scope, p, "virtualKeyboard", get_virtual_keyboard)?;
    crate::webidl::define_readonly_accessor(scope, p, "wakeLock", get_wake_lock)?;
    super::navigator_device_memory_property::define(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "userAgentData", get_user_agent_data)?;
    crate::webidl::define_readonly_accessor(scope, p, "locks", get_locks)?;
    crate::webidl::define_readonly_accessor(scope, p, "storage", get_storage)?;
    crate::webidl::define_readonly_accessor(scope, p, "gpu", get_gpu)?;
    super::navigator_contacts_property::define(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "login", get_login)?;
    crate::webidl::define_readonly_accessor(scope, p, "ink", get_ink)?;
    crate::webidl::define_readonly_accessor(scope, p, "mediaCapabilities", get_media_capabilities)?;
    crate::webidl::define_readonly_accessor(scope, p, "permissions", get_permissions)?;
    crate::webidl::define_readonly_accessor(scope, p, "devicePosture", get_device_posture)?;
    crate::webidl::define_readonly_accessor(scope, p, "hid", get_hid)?;
    crate::webidl::define_readonly_accessor(scope, p, "mediaSession", get_media_session)?;
    crate::webidl::define_readonly_accessor(scope, p, "presentation", get_presentation)?;
    crate::webidl::define_readonly_accessor(scope, p, "serial", get_serial)?;
    crate::webidl::define_readonly_accessor(scope, p, "usb", get_usb)?;
    crate::webidl::define_readonly_accessor(scope, p, "xr", get_xr)?;
    crate::webidl::define_readonly_accessor(scope, p, "storageBuckets", get_storage_buckets)?;
    crate::webidl::define_method(scope, p, "adAuctionComponents", 1, ad_auction_components)?;
    crate::webidl::define_method(scope, p, "runAdAuction", 1, run_ad_auction)?;
    crate::webidl::define_method(
        scope,
        p,
        "canLoadAdAuctionFencedFrame",
        0,
        can_load_ad_auction_fenced_frame,
    )?;
    crate::webidl::define_method(scope, p, "canShare", 0, can_share)?;
    crate::webidl::define_method(scope, p, "share", 0, share)?;
    crate::webidl::define_method(scope, p, "clearAppBadge", 0, clear_app_badge)?;
    crate::webidl::define_method(scope, p, "getBattery", 0, get_battery)?;
    crate::webidl::define_method(scope, p, "getUserMedia", 3, get_user_media)?;
    crate::webidl::define_method(scope, p, "requestMIDIAccess", 0, request_midi_access)?;
    crate::webidl::define_method(
        scope,
        p,
        "requestMediaKeySystemAccess",
        2,
        request_media_key_system_access,
    )?;
    crate::webidl::define_method(scope, p, "setAppBadge", 0, set_app_badge)?;
    crate::webidl::define_method(scope, p, "webkitGetUserMedia", 3, webkit_get_user_media)?;
    crate::webidl::define_method(
        scope,
        p,
        "clearOriginJoinedAdInterestGroups",
        1,
        clear_origin_joined_ad_interest_groups,
    )?;
    crate::webidl::define_method(scope, p, "createAuctionNonce", 0, create_auction_nonce)?;
    crate::webidl::define_method(scope, p, "joinAdInterestGroup", 1, join_ad_interest_group)?;
    crate::webidl::define_method(scope, p, "leaveAdInterestGroup", 0, leave_ad_interest_group)?;
    crate::webidl::define_method(
        scope,
        p,
        "updateAdInterestGroups",
        0,
        update_ad_interest_groups,
    )?;
    crate::webidl::define_method(
        scope,
        p,
        "deprecatedReplaceInURN",
        2,
        deprecated_replace_in_urn,
    )?;
    crate::webidl::define_method(scope, p, "deprecatedURNToURL", 1, deprecated_urn_to_url)?;
    crate::webidl::define_method(
        scope,
        p,
        "getInstalledRelatedApps",
        0,
        get_installed_related_apps,
    )?;
    crate::webidl::define_method(
        scope,
        p,
        "getInterestGroupAdAuctionData",
        1,
        get_interest_group_ad_auction_data,
    )?;
    super::navigator_cookie_deprecation_label_property::define(scope, p)?;
    crate::webidl::define_method(
        scope,
        p,
        "registerProtocolHandler",
        2,
        register_protocol_handler,
    )?;
    crate::webidl::define_method(
        scope,
        p,
        "unregisterProtocolHandler",
        2,
        unregister_protocol_handler,
    )?;
    let version = crate::browser_surface::current_version(scope);
    crate::browser_surface::reorder_string_properties(
        scope,
        p,
        crate::browser_surface::navigator_names(version),
        "Navigator.prototype",
    )?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NavigatorStore>()
        .ok_or_else(|| "Navigator state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let navigator = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, navigator, p.into()) != Some(true) {
        return Err("cannot create Navigator".to_owned());
    }
    let (has_been_active, is_active) = {
        let fingerprint = crate::fingerprint::navigator(scope);
        (
            fingerprint.user_activation_has_been_active,
            fingerprint.user_activation_is_active,
        )
    };
    let scheduling = super::scheduling::create(scope)?;
    let user_activation = super::user_activation::create(scope, has_been_active, is_active)?;
    let permissions = super::permissions::create(scope)?;
    let plugins = super::plugin_array::create(scope)?;
    let mime_types = super::mime_type_array::create(scope, plugins)?;
    let connection = super::network_information::create(scope)?;
    let protected_audience = super::protected_audience::create(scope)?;
    let bluetooth = super::bluetooth::create(scope)?;
    let clipboard = super::clipboard::create(scope)?;
    let credentials = super::credentials_container::create(scope)?;
    let keyboard = super::keyboard::create(scope)?;
    let managed = super::navigator_managed_data::create(scope)?;
    let media_devices = super::media_devices::create(scope)?;
    let service_worker = super::service_worker_container::create(scope)?;
    let virtual_keyboard = super::virtual_keyboard::create(scope)?;
    let wake_lock = super::wake_lock::create(scope)?;
    let user_agent_data = super::navigator_ua_data::create(scope)?;
    let locks = super::lock_manager::create(scope)?;
    let storage = super::storage_manager::create(scope)?;
    let gpu = super::gpu::create(scope)?;
    let login = super::navigator_login::create(scope)?;
    let configured_languages = crate::fingerprint::navigator(scope).languages.clone();
    let languages = v8::Array::new(scope, configured_languages.len() as i32);
    for (index, language) in configured_languages.iter().enumerate() {
        set_string_index(scope, languages, index as u32, language);
    }
    let _ = languages.set_integrity_level(scope, v8::IntegrityLevel::Frozen);
    let geolocation = super::geolocation::create(scope)?;
    let temporary_storage = service_object(scope, "DeprecatedStorageQuota");
    let persistent_storage = service_object(scope, "DeprecatedStorageQuota");
    let overlay = service_object(scope, "WindowControlsOverlay");
    let ink = super::ink::create(scope)?;
    let media_capabilities = super::media_capabilities::create(scope)?;
    let media_session = super::media_session::create(scope)?;
    let device_posture = super::device_posture::create(scope)?;
    let hid = super::hid::create(scope)?;
    let presentation = super::presentation::create(scope)?;
    let serial = super::serial::create(scope)?;
    let usb = super::usb::create(scope)?;
    let xr = super::xr_system::create(scope)?;
    let storage_buckets = super::storage_bucket_manager::create(scope)?;
    let contacts = super::contacts_manager::create(scope)?;
    let model_context = super::model_context::create(scope)?;
    let cookie_deprecation_label = super::cookie_deprecation_label::create(scope)?;
    let record = NavigatorRecord {
        scheduling: v8::Global::new(scope, scheduling),
        user_activation: v8::Global::new(scope, user_activation),
        geolocation: v8::Global::new(scope, geolocation),
        temporary_storage: v8::Global::new(scope, temporary_storage),
        persistent_storage: v8::Global::new(scope, persistent_storage),
        window_controls_overlay: v8::Global::new(scope, overlay),
        languages: v8::Global::new(scope, languages),
        plugins: v8::Global::new(scope, plugins),
        mime_types: v8::Global::new(scope, mime_types),
        connection: v8::Global::new(scope, connection),
        protected_audience: v8::Global::new(scope, protected_audience),
        bluetooth: v8::Global::new(scope, bluetooth),
        clipboard: v8::Global::new(scope, clipboard),
        credentials: v8::Global::new(scope, credentials),
        keyboard: v8::Global::new(scope, keyboard),
        managed: v8::Global::new(scope, managed),
        media_devices: v8::Global::new(scope, media_devices),
        service_worker: v8::Global::new(scope, service_worker),
        virtual_keyboard: v8::Global::new(scope, virtual_keyboard),
        wake_lock: v8::Global::new(scope, wake_lock),
        user_agent_data: v8::Global::new(scope, user_agent_data),
        locks: v8::Global::new(scope, locks),
        storage: v8::Global::new(scope, storage),
        gpu: v8::Global::new(scope, gpu),
        login: v8::Global::new(scope, login),
        ink: v8::Global::new(scope, ink),
        media_capabilities: v8::Global::new(scope, media_capabilities),
        permissions: v8::Global::new(scope, permissions),
        device_posture: v8::Global::new(scope, device_posture),
        hid: v8::Global::new(scope, hid),
        media_session: v8::Global::new(scope, media_session),
        presentation: v8::Global::new(scope, presentation),
        serial: v8::Global::new(scope, serial),
        usb: v8::Global::new(scope, usb),
        xr: v8::Global::new(scope, xr),
        storage_buckets: v8::Global::new(scope, storage_buckets),
        contacts: v8::Global::new(scope, contacts),
        model_context: v8::Global::new(scope, model_context),
        cookie_deprecation_label: v8::Global::new(scope, cookie_deprecation_label),
        vibration_pattern: Vec::new(),
    };
    scope
        .get_slot_mut::<NavigatorStore>()
        .ok_or_else(|| "Navigator state was not prepared".to_owned())?
        .records
        .insert(navigator.get_identity_hash().get(), record);
    Ok(navigator)
}
fn service_object<'s>(scope: &v8::PinScope<'s, '_>, name: &str) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    let tag = v8::Symbol::get_to_string_tag(scope);
    if let Some(value) = v8::String::new(scope, name) {
        let _ = object.define_own_property(
            scope,
            tag.into(),
            value.into(),
            v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
        );
    }
    object
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Navigator': Illegal constructor",
    )
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigatorRecord> {
    scope
        .get_slot::<NavigatorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn valid_this(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    let valid = scope.get_slot::<NavigatorStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    valid
}
pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: &str,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(s) = v8::String::new(scope, value) {
        r.set(s.into())
    }
}
fn return_object(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&NavigatorRecord) -> v8::Global<v8::Object>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &select(&v)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) enum AndroidObjectProperty {
    Contacts,
    ModelContext,
    CookieDeprecationLabel,
}

pub(crate) fn get_android_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
    property: AndroidObjectProperty,
) {
    return_object(scope, arguments, result, |record| match property {
        AndroidObjectProperty::Contacts => record.contacts.clone(),
        AndroidObjectProperty::ModelContext => record.model_context.clone(),
        AndroidObjectProperty::CookieDeprecationLabel => record.cookie_deprecation_label.clone(),
    });
}
fn get_scheduling(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.scheduling.clone())
}
fn get_user_activation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.user_activation.clone())
}
fn get_geolocation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.geolocation.clone())
}
fn get_temporary_storage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.temporary_storage.clone())
}
fn get_persistent_storage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.persistent_storage.clone())
}
fn get_window_controls_overlay(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.window_controls_overlay.clone())
}
fn get_plugins(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.plugins.clone())
}
fn get_mime_types(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.mime_types.clone())
}
fn get_connection(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.connection.clone())
}
fn get_deprecated_run_ad_auction_enforces_k_anonymity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Boolean::new(s, true).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_protected_audience(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.protected_audience.clone())
}
fn get_bluetooth(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.bluetooth.clone())
}
fn get_clipboard(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.clipboard.clone())
}
fn get_credentials(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.credentials.clone())
}
fn get_keyboard(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.keyboard.clone())
}
fn get_managed(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.managed.clone())
}
fn get_media_devices(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.media_devices.clone())
}
fn get_service_worker(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.service_worker.clone())
}
fn get_virtual_keyboard(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.virtual_keyboard.clone())
}
fn get_wake_lock(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.wake_lock.clone())
}
fn get_user_agent_data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.user_agent_data.clone())
}
fn get_locks(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.locks.clone())
}
fn get_storage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.storage.clone())
}
fn get_gpu(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.gpu.clone())
}
fn get_login(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.login.clone())
}
fn get_ink(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.ink.clone())
}
fn get_media_capabilities(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.media_capabilities.clone())
}
fn get_permissions(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.permissions.clone())
}
fn get_device_posture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.device_posture.clone())
}
fn get_hid(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.hid.clone())
}
fn get_media_session(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.media_session.clone())
}
fn get_presentation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.presentation.clone())
}
fn get_serial(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.serial.clone())
}
fn get_usb(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.usb.clone())
}
fn get_xr(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.xr.clone())
}
fn get_storage_buckets(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_object(s, a, r, |v| v.storage_buckets.clone())
}
pub(crate) fn languages_object(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Array>> {
    record(scope, object).map(|record| record.languages)
}
fn get_gamepads(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let configured = crate::fingerprint::edge(scope)
        .hardware_devices
        .gamepads
        .clone();
    let length = configured
        .iter()
        .map(|gamepad| gamepad.index.saturating_add(1))
        .max()
        .unwrap_or(0)
        .max(4);
    let array = v8::Array::new(scope, length as i32);
    for index in 0..length {
        let _ = array.set_index(scope, index, v8::null(scope).into());
    }
    for gamepad in configured {
        if let Ok(object) = super::gamepad::create(
            scope,
            &gamepad.id,
            gamepad.index,
            gamepad.connected,
            &gamepad.mapping,
            &gamepad.axes,
            &gamepad.buttons,
        ) {
            let _ = array.set_index(scope, gamepad.index, object.into());
        }
    }
    r.set(array.into())
}
fn java_enabled(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(v8::Boolean::new(scope, false).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn send_beacon(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let address = crate::webidl::value_to_string(scope, a.get(0));
    if !(address.starts_with("http://") || address.starts_with("https://")) {
        crate::webidl::throw_type_error(scope, "Beacons are only supported over HTTP(S)");
        return;
    }
    r.set(v8::Boolean::new(scope, false).into())
}
fn vibrate(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let mut pattern = Vec::new();
    if let Ok(sequence) = v8::Local::<v8::Object>::try_from(a.get(0)) {
        let length_key = v8::String::new(scope, "length").unwrap();
        let length = sequence
            .get(scope, length_key.into())
            .and_then(|value| value.uint32_value(scope))
            .unwrap_or(0);
        for index in 0..length {
            pattern.push(
                sequence
                    .get_index(scope, index)
                    .and_then(|value| value.uint32_value(scope))
                    .unwrap_or(0)
                    .min(10000),
            );
        }
    } else {
        pattern.push(a.get(0).uint32_value(scope).unwrap_or(0).min(10000));
    }
    if let Some(v) = scope
        .get_slot_mut::<NavigatorStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.vibration_pattern = pattern;
        r.set(v8::Boolean::new(scope, true).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn ensure_navigator(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    if record(scope, object).is_some() {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}
fn ensure_navigator_promise(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    method: &str,
    result: &mut v8::ReturnValue<'_>,
) -> bool {
    if record(scope, object).is_some() {
        return true;
    }
    let message = format!("Failed to execute '{method}' on 'Navigator': Illegal invocation");
    if let Some(promise) = crate::webidl::rejected_type_error_promise(scope, &message) {
        result.set(promise.into());
    }
    false
}
fn resolve_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into())
    }
}
fn resolve_undefined(scope: &mut v8::PinScope<'_, '_>, result: v8::ReturnValue<'_>) {
    let undefined = v8::undefined(scope);
    resolve_value(scope, undefined.into(), result)
}
fn ad_auction_components(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator(scope, arguments.this()) {
        return;
    }
    let components = v8::Array::new(scope, 0);
    result.set(components.into());
}
fn run_ad_auction(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(scope, arguments.this(), "runAdAuction", &mut result) {
        let null = v8::null(scope);
        resolve_value(scope, null.into(), result)
    }
}
fn can_load_ad_auction_fenced_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, false).into())
    }
}
fn can_share(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator(scope, arguments.this()) {
        let supported = arguments.get(0).is_object();
        result.set(v8::Boolean::new(scope, supported).into())
    }
}
fn share(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(scope, arguments.this(), "share", &mut result) {
        resolve_undefined(scope, result)
    }
}
fn clear_app_badge(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(scope, arguments.this(), "clearAppBadge", &mut result) {
        resolve_undefined(scope, result)
    }
}
fn get_battery(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator_promise(scope, arguments.this(), "getBattery", &mut result) {
        return;
    }
    match super::battery_manager::create(scope) {
        Ok(battery) => resolve_value(scope, battery.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn deliver_user_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
) {
    if !ensure_navigator(scope, arguments.this()) {
        return;
    }
    let stream = match super::media_stream::create_with_tracks(scope, &[]) {
        Ok(stream) => stream,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(1)) {
        let receiver = scope.get_current_context().global(scope);
        let values = [stream.into()];
        let _ = callback.call(scope, receiver.into(), &values);
    }
}
fn get_user_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    deliver_user_media(scope, arguments)
}
fn request_midi_access(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator_promise(scope, arguments.this(), "requestMIDIAccess", &mut result) {
        return;
    }
    match super::midi_access::create(scope) {
        Ok(access) => resolve_value(scope, access.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn request_media_key_system_access(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator_promise(
        scope,
        arguments.this(),
        "requestMediaKeySystemAccess",
        &mut result,
    ) {
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "requestMediaKeySystemAccess requires a key system and configurations.",
        );
        return;
    }
    let key_system = crate::webidl::value_to_string(scope, arguments.get(0));
    let configuration = v8::Local::<v8::Array>::try_from(arguments.get(1))
        .ok()
        .and_then(|values| values.get_index(scope, 0))
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .unwrap_or_else(|| v8::Object::new(scope));
    match super::media_key_system_access::create(scope, key_system, configuration) {
        Ok(access) => resolve_value(scope, access.into(), result),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
fn set_app_badge(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator_promise(scope, arguments.this(), "setAppBadge", &mut result) {
        return;
    }
    if !arguments.get(0).is_undefined() {
        let _ = arguments.get(0).number_value(scope);
    }
    resolve_undefined(scope, result)
}
fn webkit_get_user_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    deliver_user_media(scope, arguments)
}
fn clear_origin_joined_ad_interest_groups(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(
        scope,
        arguments.this(),
        "clearOriginJoinedAdInterestGroups",
        &mut result,
    ) {
        resolve_undefined(scope, result)
    }
}
fn create_auction_nonce(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator_promise(scope, arguments.this(), "createAuctionNonce", &mut result) {
        return;
    }
    let nonce = v8::String::new(scope, "00000000-0000-4000-8000-000000000001")
        .expect("valid auction nonce");
    resolve_value(scope, nonce.into(), result)
}
fn join_ad_interest_group(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(scope, arguments.this(), "joinAdInterestGroup", &mut result) {
        resolve_undefined(scope, result)
    }
}
fn leave_ad_interest_group(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(scope, arguments.this(), "leaveAdInterestGroup", &mut result) {
        resolve_undefined(scope, result)
    }
}
fn update_ad_interest_groups(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if ensure_navigator(scope, arguments.this()) {
        resolve_undefined(scope, result)
    }
}
fn deprecated_replace_in_urn(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(
        scope,
        arguments.this(),
        "deprecatedReplaceInURN",
        &mut result,
    ) {
        resolve_undefined(scope, result)
    }
}
fn deprecated_urn_to_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator_promise(scope, arguments.this(), "deprecatedURNToURL", &mut result) {
        return;
    }
    let urn = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = v8::String::new(scope, &urn).expect("valid URN");
    resolve_value(scope, value.into(), result)
}
fn get_installed_related_apps(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if ensure_navigator_promise(
        scope,
        arguments.this(),
        "getInstalledRelatedApps",
        &mut result,
    ) {
        let applications = v8::Array::new(scope, 0);
        resolve_value(scope, applications.into(), result)
    }
}
fn get_interest_group_ad_auction_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !ensure_navigator_promise(
        scope,
        arguments.this(),
        "getInterestGroupAdAuctionData",
        &mut result,
    ) {
        return;
    }
    let data = v8::Object::new(scope);
    let request_id_key = v8::String::new(scope, "requestId").expect("short key");
    let request_key = v8::String::new(scope, "request").expect("short key");
    let request_id = v8::String::new(scope, "edge-sandbox-request").expect("short value");
    let request = v8::ArrayBuffer::new(scope, 0);
    let _ = data.set(scope, request_id_key.into(), request_id.into());
    let _ = data.set(scope, request_key.into(), request.into());
    resolve_value(scope, data.into(), result)
}
fn register_protocol_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !ensure_navigator(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "A scheme and URL are required.");
        return;
    }
    let scheme = crate::webidl::value_to_string(scope, arguments.get(0));
    let address = crate::webidl::value_to_string(scope, arguments.get(1));
    if scheme.is_empty() || !address.contains("%s") {
        crate::webidl::throw_type_error(scope, "The handler URL must contain %s.");
    }
}
fn unregister_protocol_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !ensure_navigator(scope, arguments.this()) {
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "A scheme and URL are required.");
    }
}
fn set_string_index(
    scope: &v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    index: u32,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        let _ = array.set_index(scope, index, value.into());
    }
}
