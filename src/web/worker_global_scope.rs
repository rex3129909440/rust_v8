use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RealmKind {
    Dedicated,
    Shared,
    Service,
}

#[derive(Clone, Copy)]
pub(crate) enum RealmOwner {
    Dedicated(i32),
    Shared(u64),
    Service(i32),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ServiceHandlerKind {
    Activate,
    BackgroundFetchAbort,
    BackgroundFetchClick,
    BackgroundFetchFail,
    BackgroundFetchSuccess,
    CanMakePayment,
    ContentDelete,
    CookieChange,
    Fetch,
    Install,
    Message,
    MessageError,
    NotificationClick,
    NotificationClose,
    PaymentRequest,
    PeriodicSync,
    Push,
    PushSubscriptionChange,
    Sync,
}

#[derive(Clone)]
enum TimerCallback {
    Function(v8::Global<v8::Function>),
    Source(String),
}

#[derive(Clone)]
struct TimerRecord {
    callback: TimerCallback,
    arguments: Vec<v8::Global<v8::Value>>,
    delay_ms: f64,
    due_ms: f64,
    nesting_level: u32,
}

#[derive(Clone)]
pub(crate) struct WorkerRealmRecord {
    pub(crate) context: v8::Global<v8::Context>,
    pub(crate) global: v8::Global<v8::Object>,
    pub(crate) global_target: Option<v8::Global<v8::Object>>,
    pub(crate) owner: RealmOwner,
    pub(crate) kind: RealmKind,
    pub(crate) url: String,
    pub(crate) name: String,
    pub(crate) module: bool,
    pub(crate) closed: bool,
    pub(crate) location: Option<v8::Global<v8::Object>>,
    pub(crate) navigator: Option<v8::Global<v8::Object>>,
    pub(crate) crypto: Option<v8::Global<v8::Object>>,
    pub(crate) performance: Option<v8::Global<v8::Object>>,
    pub(crate) scheduler: Option<v8::Global<v8::Object>>,
    pub(crate) trusted_types: Option<v8::Global<v8::Object>>,
    pub(crate) caches: Option<v8::Global<v8::Object>>,
    pub(crate) indexed_db: Option<v8::Global<v8::Object>>,
    pub(crate) fonts: Option<v8::Global<v8::Object>>,
    pub(crate) origin: String,
    module_urls: HashMap<i32, String>,
    modules: HashMap<String, v8::Global<v8::Module>>,
    pub(crate) onerror: Option<v8::Global<v8::Value>>,
    pub(crate) onlanguagechange: Option<v8::Global<v8::Value>>,
    pub(crate) onoffline: Option<v8::Global<v8::Value>>,
    pub(crate) ononline: Option<v8::Global<v8::Value>>,
    pub(crate) onunhandledrejection: Option<v8::Global<v8::Value>>,
    pub(crate) onrejectionhandled: Option<v8::Global<v8::Value>>,
    pub(crate) onmessage: Option<v8::Global<v8::Value>>,
    pub(crate) onmessageerror: Option<v8::Global<v8::Value>>,
    pub(crate) onconnect: Option<v8::Global<v8::Value>>,
    pub(crate) onrtctransform: Option<v8::Global<v8::Value>>,
    pub(crate) service_handlers: HashMap<ServiceHandlerKind, v8::Global<v8::Value>>,
    pub(crate) service_registration: Option<v8::Global<v8::Object>>,
    pub(crate) service_worker: Option<v8::Global<v8::Object>>,
    pub(crate) service_clients: Option<v8::Global<v8::Object>>,
    pub(crate) outgoing: Vec<super::worker_structured_clone::SerializedMessage>,
    next_timer_id: i32,
    timeouts: HashMap<i32, TimerRecord>,
    intervals: HashMap<i32, TimerRecord>,
    animation_frames: HashMap<i32, v8::Global<v8::Function>>,
    animation_frame_due_ms: Option<f64>,
    running_timer_nesting_level: u32,
}

pub(crate) struct WorkerRealmStore {
    next_id: i32,
    records: HashMap<i32, WorkerRealmRecord>,
}

impl Default for WorkerRealmStore {
    fn default() -> Self {
        Self {
            next_id: 1,
            records: HashMap::new(),
        }
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WorkerRealmStore::default());
    isolate.set_host_import_module_dynamically_callback(dynamic_import);
}

pub(crate) fn enable_native_trace_for_existing_realms(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let realms = scope
        .get_slot::<WorkerRealmStore>()
        .map(|store| {
            store
                .records
                .iter()
                .map(|(id, record)| {
                    (
                        *id,
                        record.context.clone(),
                        record.global.clone(),
                        record.global_target.clone(),
                        record.kind,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for (id, context, global, target, kind) in realms {
        let context = v8::Local::new(scope, &context);
        let child_scope = &mut v8::ContextScope::new(scope, context);
        let global = v8::Local::new(child_scope, &global);
        let label = match kind {
            RealmKind::Dedicated => format!("worker[{id}]"),
            RealmKind::Shared => format!("sharedWorker[{id}]"),
            RealmKind::Service => format!("serviceWorker[{id}]"),
        };
        crate::trace::label_native_value(child_scope, global.into(), &label);
        if let Some(target) = target {
            let target = v8::Local::new(child_scope, &target);
            crate::trace::label_native_value(child_scope, target.into(), &label);
        }
    }
    Ok(())
}

pub(crate) fn disable_native_trace_for_existing_realms(_: &mut v8::OwnedIsolate) {}

pub(crate) fn create(
    scope: &mut v8::PinScope<'_, '_>,
    owner: RealmOwner,
    kind: RealmKind,
    url: String,
    name: String,
    module: bool,
) -> Result<i32, String> {
    let origin = current_record(scope)
        .map(|record| record.origin)
        .unwrap_or_else(|| {
            let context = scope.get_entered_or_microtask_context();
            let window = context.global(scope);
            super::html_i_frame_element::origin_for_window(scope, window)
        });
    let realm_id = {
        let store = scope
            .get_slot_mut::<WorkerRealmStore>()
            .ok_or_else(|| "Worker realm state was not prepared".to_owned())?;
        let id = store.next_id;
        store.next_id = store.next_id.saturating_add(1).max(1);
        id
    };
    let context = v8::Context::new(scope, v8::ContextOptions::default());
    let global = context.global(scope);
    let record = WorkerRealmRecord {
        context: v8::Global::new(scope, context),
        global: v8::Global::new(scope, global),
        global_target: None,
        owner,
        kind,
        url,
        name,
        module,
        closed: false,
        location: None,
        navigator: None,
        crypto: None,
        performance: None,
        scheduler: None,
        trusted_types: None,
        caches: None,
        indexed_db: None,
        fonts: None,
        origin,
        module_urls: HashMap::new(),
        modules: HashMap::new(),
        onerror: None,
        onlanguagechange: None,
        onoffline: None,
        ononline: None,
        onunhandledrejection: None,
        onrejectionhandled: None,
        onmessage: None,
        onmessageerror: None,
        onconnect: None,
        onrtctransform: None,
        service_handlers: HashMap::new(),
        service_registration: None,
        service_worker: None,
        service_clients: None,
        outgoing: Vec::new(),
        next_timer_id: 1,
        timeouts: HashMap::new(),
        intervals: HashMap::new(),
        animation_frames: HashMap::new(),
        animation_frame_due_ms: None,
        running_timer_nesting_level: 0,
    };
    scope
        .get_slot_mut::<WorkerRealmStore>()
        .expect("Worker realm state")
        .records
        .insert(realm_id, record);

    let install_result = {
        let realm_scope = &mut v8::ContextScope::new(scope, context);
        let global = realm_scope.get_current_context().global(realm_scope);
        let trace_target = global
            .get_prototype(realm_scope)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or_else(|| "Worker global target is unavailable".to_owned())?;
        mirror_worker_intrinsics_to_target(realm_scope, trace_target, kind)?;
        super::console_edge::install(realm_scope)?;
        if kind != RealmKind::Dedicated {
            mirror_global_to_target(realm_scope, trace_target, "console")?;
        }
        install_common_globals(realm_scope)?;
        super::worker_realm_interfaces::install(realm_scope)?;
        if kind != RealmKind::Dedicated {
            mirror_realm_local_globals(realm_scope, trace_target)?;
        }
        let worker_scope = install_scope_prototypes(realm_scope, kind, trace_target)?;
        if kind != RealmKind::Dedicated {
            mirror_global_to_target(realm_scope, trace_target, "WorkerGlobalScope")?;
            match kind {
                RealmKind::Dedicated => unreachable!("dedicated Worker order is installed later"),
                RealmKind::Shared => {
                    mirror_global_to_target(realm_scope, trace_target, "SharedWorkerGlobalScope")?
                }
                RealmKind::Service => {
                    mirror_global_to_target(realm_scope, trace_target, "ServiceWorkerGlobalScope")?
                }
            }
        }
        let location = super::worker_location::create(realm_scope)?;
        let navigator = super::worker_navigator::create(realm_scope)?;
        let crypto = super::crypto::create(realm_scope)?;
        let performance = super::performance::create(realm_scope, false)?;
        let scheduler = super::scheduler::create(realm_scope)?;
        let trusted_types = super::trusted_type_policy_factory::create(realm_scope)?;
        let caches = super::cache_storage::create(realm_scope)?;
        let indexed_db = super::idb_factory::create(realm_scope)?;
        let fonts = super::font_face_set::create(realm_scope)?;
        crate::webidl::define_global(realm_scope, "WorkerLocation", location.0.into())?;
        crate::webidl::define_global(realm_scope, "WorkerNavigator", navigator.0.into())?;
        if kind != RealmKind::Dedicated {
            mirror_global_to_target(realm_scope, trace_target, "WorkerLocation")?;
            mirror_global_to_target(realm_scope, trace_target, "WorkerNavigator")?;
            mirror_global_to_target(realm_scope, trace_target, "FileReaderSync")?;
        }
        if kind == RealmKind::Dedicated {
            super::worker::install_in_worker_realm(realm_scope)?;
            super::rtc_rtp_script_transformer::install_in_worker_realm(realm_scope)?;
            super::rtc_transform_event::install_in_worker_realm(realm_scope)?;
            super::file_system_sync_access_handle::install_in_worker_realm(realm_scope)?;
            super::file_system_file_handle::install_in_worker_realm(realm_scope)?;
            super::worker_global_edge_order::install_dedicated(realm_scope, trace_target)?;
        }
        if kind == RealmKind::Service {
            super::extendable_event::install_in_service_worker_realm(realm_scope)?;
            super::fetch_event::install_in_service_worker_realm(realm_scope)?;
            mirror_global_to_target(realm_scope, trace_target, "ExtendableEvent")?;
            mirror_global_to_target(realm_scope, trace_target, "FetchEvent")?;
        }
        if crate::webidl::set_platform_prototype(realm_scope, trace_target, worker_scope.into())
            != Some(true)
        {
            return Err("cannot attach WorkerGlobalScope target prototype".to_owned());
        }
        super::event_target::attach(realm_scope, global);
        super::event_target::attach_alias(realm_scope, trace_target, global);
        crate::determinism::install(realm_scope)?;
        if crate::trace::is_enabled(realm_scope) {
            let label = match kind {
                RealmKind::Dedicated => format!("worker[{realm_id}]"),
                RealmKind::Shared => format!("sharedWorker[{realm_id}]"),
                RealmKind::Service => format!("serviceWorker[{realm_id}]"),
            };
            crate::trace::reserve_prototype_property_label_from_global(
                realm_scope,
                "MessagePort",
                "postMessage",
                &format!("{label}.MessagePort.prototype.postMessage"),
            )?;
            crate::trace::label_native_value(realm_scope, global.into(), &label);
            crate::trace::label_native_value(realm_scope, trace_target.into(), &label);
        }
        Ok::<_, String>((
            location.1,
            navigator.1,
            crypto,
            performance,
            scheduler,
            trusted_types,
            caches,
            indexed_db,
            fonts,
            trace_target,
        ))
    };
    match install_result {
        Ok((
            location,
            navigator,
            crypto,
            performance,
            scheduler,
            trusted_types,
            caches,
            indexed_db,
            fonts,
            global_target,
        )) => {
            let location = v8::Global::new(scope, location);
            let navigator = v8::Global::new(scope, navigator);
            let crypto = v8::Global::new(scope, crypto);
            let performance = v8::Global::new(scope, performance);
            let scheduler = v8::Global::new(scope, scheduler);
            let trusted_types = v8::Global::new(scope, trusted_types);
            let caches = v8::Global::new(scope, caches);
            let indexed_db = v8::Global::new(scope, indexed_db);
            let fonts = v8::Global::new(scope, fonts);
            let global_target = v8::Global::new(scope, global_target);
            let record = scope
                .get_slot_mut::<WorkerRealmStore>()
                .and_then(|store| store.records.get_mut(&realm_id))
                .ok_or_else(|| "Worker realm disappeared during installation".to_owned())?;
            record.location = Some(location);
            record.navigator = Some(navigator);
            record.crypto = Some(crypto);
            record.performance = Some(performance);
            record.scheduler = Some(scheduler);
            record.trusted_types = Some(trusted_types);
            record.caches = Some(caches);
            record.indexed_db = Some(indexed_db);
            record.fonts = Some(fonts);
            record.global_target = Some(global_target);
            Ok(realm_id)
        }
        Err(error) => {
            scope
                .get_slot_mut::<WorkerRealmStore>()
                .expect("Worker realm state")
                .records
                .remove(&realm_id);
            Err(error)
        }
    }
}

fn install_scope_prototypes<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    kind: RealmKind,
    global_target: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let event_target = global_function(scope, "EventTarget")?;
    let worker_scope = crate::webidl::create_function(
        scope,
        "WorkerGlobalScope",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_worker_scope_constructor,
    )?;
    crate::webidl::inherit(scope, worker_scope, event_target)?;
    let worker_prototype = crate::webidl::prototype(scope, worker_scope)?;
    crate::webidl::reset_constructor_order(scope, worker_prototype)?;
    super::worker_global_scope_self_property::define(scope, worker_prototype)?;
    super::worker_global_scope_location_property::define(scope, worker_prototype)?;
    super::worker_global_scope_onerror_property::define(scope, worker_prototype)?;
    super::worker_global_scope_onlanguagechange_property::define(scope, worker_prototype)?;
    super::worker_global_scope_navigator_property::define(scope, worker_prototype)?;
    super::worker_global_scope_onrejectionhandled_property::define(scope, worker_prototype)?;
    super::worker_global_scope_onunhandledrejection_property::define(scope, worker_prototype)?;
    super::worker_global_scope_origin_property::define(scope, worker_prototype)?;
    super::worker_global_scope_performance_property::define(scope, worker_prototype)?;
    super::worker_global_scope_trusted_types_property::define(scope, worker_prototype)?;
    super::worker_global_scope_crypto_property::define(scope, worker_prototype)?;
    super::worker_global_scope_indexed_db_property::define(scope, worker_prototype)?;
    super::worker_global_scope_fonts_property::define(scope, worker_prototype)?;
    super::worker_global_scope_create_image_bitmap::define(scope, worker_prototype)?;
    super::worker_global_scope_fetch::define(scope, worker_prototype)?;
    super::worker_global_scope_import_scripts::define(scope, worker_prototype)?;
    crate::webidl::finish_constructor(scope, worker_prototype, worker_scope)?;
    super::worker_global_scope_is_secure_context_property::define(scope, worker_prototype)?;
    super::worker_global_scope_cross_origin_isolated_property::define(scope, worker_prototype)?;
    super::worker_global_scope_scheduler_property::define(scope, worker_prototype)?;
    super::worker_global_scope_caches_property::define(scope, worker_prototype)?;
    super::worker_global_scope_atob::define(scope, worker_prototype)?;
    super::worker_global_scope_btoa::define(scope, worker_prototype)?;
    super::worker_global_queue_microtask::define(scope, worker_prototype)?;
    super::worker_global_scope_report_error::define(scope, worker_prototype)?;
    super::worker_global_scope_structured_clone::define(scope, worker_prototype)?;
    super::worker_global_clear_interval::define(scope, worker_prototype)?;
    super::worker_global_clear_timeout::define(scope, worker_prototype)?;
    super::worker_global_set_interval::define(scope, worker_prototype)?;
    super::worker_global_set_timeout::define(scope, worker_prototype)?;
    crate::webidl::define_global(scope, "WorkerGlobalScope", worker_scope.into())?;

    match kind {
        RealmKind::Dedicated => {
            let dedicated = crate::webidl::create_function(
                scope,
                "DedicatedWorkerGlobalScope",
                0,
                v8::ConstructorBehavior::Allow,
                illegal_dedicated_scope_constructor,
            )?;
            crate::webidl::inherit(scope, dedicated, worker_scope)?;
            let prototype = crate::webidl::prototype(scope, dedicated)?;
            crate::webidl::reset_constructor_order(scope, prototype)?;
            crate::webidl::define_constant(scope, prototype, "TEMPORARY", 0)?;
            crate::webidl::define_constant(scope, prototype, "PERSISTENT", 1)?;
            crate::webidl::finish_constructor(scope, prototype, dedicated)?;
            crate::webidl::define_constant(scope, dedicated.into(), "TEMPORARY", 0)?;
            crate::webidl::define_constant(scope, dedicated.into(), "PERSISTENT", 1)?;
            crate::webidl::define_global(scope, "DedicatedWorkerGlobalScope", dedicated.into())?;
            let global = scope.get_current_context().global(scope);
            install_dedicated_global_members(scope, global)?;
            Ok(prototype)
        }
        RealmKind::Shared => {
            let shared = crate::webidl::create_function(
                scope,
                "SharedWorkerGlobalScope",
                0,
                v8::ConstructorBehavior::Allow,
                illegal_shared_scope_constructor,
            )?;
            crate::webidl::inherit(scope, shared, worker_scope)?;
            let prototype = crate::webidl::prototype(scope, shared)?;
            crate::webidl::reset_constructor_order(scope, prototype)?;
            crate::webidl::finish_constructor(scope, prototype, shared)?;
            crate::webidl::define_global(scope, "SharedWorkerGlobalScope", shared.into())?;
            super::shared_worker_global_scope_name_property::define(scope, global_target)?;
            super::shared_worker_global_scope_onconnect_property::define(scope, global_target)?;
            super::shared_worker_global_scope_close::define(scope, global_target)?;
            Ok(prototype)
        }
        RealmKind::Service => {
            let service = crate::webidl::create_function(
                scope,
                "ServiceWorkerGlobalScope",
                0,
                v8::ConstructorBehavior::Allow,
                illegal_service_scope_constructor,
            )?;
            crate::webidl::inherit(scope, service, worker_scope)?;
            let prototype = crate::webidl::prototype(scope, service)?;
            crate::webidl::reset_constructor_order(scope, prototype)?;
            crate::webidl::finish_constructor(scope, prototype, service)?;
            crate::webidl::define_global(scope, "ServiceWorkerGlobalScope", service.into())?;
            super::service_worker_global_scope_clients_property::define(scope, global_target)?;
            super::service_worker_global_scope_registration_property::define(scope, global_target)?;
            super::service_worker_global_scope_service_worker_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_skip_waiting::define(scope, global_target)?;
            super::service_worker_global_scope_onactivate_property::define(scope, global_target)?;
            super::service_worker_global_scope_onbackgroundfetchabort_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onbackgroundfetchclick_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onbackgroundfetchfail_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onbackgroundfetchsuccess_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_oncanmakepayment_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_oncontentdelete_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_oncookiechange_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onfetch_property::define(scope, global_target)?;
            super::service_worker_global_scope_oninstall_property::define(scope, global_target)?;
            super::service_worker_global_scope_onmessage_property::define(scope, global_target)?;
            super::service_worker_global_scope_onmessageerror_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onnotificationclick_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onnotificationclose_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onpaymentrequest_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onperiodicsync_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onpush_property::define(scope, global_target)?;
            super::service_worker_global_scope_onpushsubscriptionchange_property::define(
                scope,
                global_target,
            )?;
            super::service_worker_global_scope_onsync_property::define(scope, global_target)?;
            Ok(prototype)
        }
    }
}

pub(crate) fn install_dedicated_global_members(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    super::dedicated_worker_global_scope_name_property::define(scope, object)?;
    super::dedicated_worker_global_scope_onmessage_property::define(scope, object)?;
    super::dedicated_worker_global_scope_onmessageerror_property::define(scope, object)?;
    super::worker_global_cancel_animation_frame::define(scope, object)?;
    super::dedicated_worker_global_scope_close::define(scope, object)?;
    super::dedicated_worker_global_scope_post_message::define(scope, object)?;
    super::worker_global_request_animation_frame::define(scope, object)?;
    super::dedicated_worker_global_scope_onrtctransform_property::define(scope, object)?;
    super::dedicated_worker_global_scope_webkit_request_file_system::define(scope, object)?;
    super::dedicated_worker_global_scope_webkit_request_file_system_sync::define(scope, object)?;
    super::dedicated_worker_global_scope_webkit_resolve_local_file_system_sync_url::define(
        scope, object,
    )?;
    super::dedicated_worker_global_scope_webkit_resolve_local_file_system_url::define(scope, object)
}

fn install_common_globals(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    super::event_target::install(scope)?;
    super::event::install(scope)?;
    super::custom_event::install(scope)?;
    super::message_event::install(scope)?;
    super::error_event::install(scope)?;
    super::promise_rejection_event::install(scope)?;
    super::dom_exception::install(scope)?;
    super::abort_signal::install(scope)?;
    super::abort_controller::install(scope)?;
    super::message_port::install(scope)?;
    super::message_channel::install(scope)?;
    super::broadcast_channel::install(scope)?;
    super::url::install_standard_name(scope)?;
    super::url_search_params::install_global(scope)?;
    super::url_pattern::install(scope)?;
    super::blob::install(scope)?;
    super::file::install(scope)?;
    super::file_reader::install(scope)?;
    super::file_reader_sync::install_in_worker_realm(scope)?;
    super::text_encoder::install(scope)?;
    super::text_decoder::install(scope)?;
    super::text_encoder_stream::install(scope)?;
    super::text_decoder_stream::install(scope)?;
    super::headers::install(scope)?;
    super::request::install(scope)?;
    super::response::install(scope)?;
    super::form_data::install(scope)?;
    super::readable_stream::install(scope)?;
    super::readable_stream_default_reader::install(scope)?;
    super::readable_stream_default_controller::install(scope)?;
    super::readable_stream_byob_request::install(scope)?;
    super::readable_stream_byob_reader::install(scope)?;
    super::readable_byte_stream_controller::install(scope)?;
    super::writable_stream::install(scope)?;
    super::writable_stream_default_writer::install(scope)?;
    super::writable_stream_default_controller::install(scope)?;
    super::transform_stream::install(scope)?;
    super::transform_stream_default_controller::install(scope)?;
    super::compression_stream::install(scope)?;
    super::decompression_stream::install(scope)?;
    super::byte_length_queuing_strategy::install(scope)?;
    super::count_queuing_strategy::install(scope)?;
    super::web_socket::install(scope)?;
    super::web_socket_stream::install(scope)?;
    super::web_socket_error::install(scope)?;
    super::webgl_vertex_array_object::install(scope)?;
    super::webgl_uniform_location::install(scope)?;
    super::webgl_transform_feedback::install(scope)?;
    super::webgl_texture::install(scope)?;
    super::webgl_sync::install(scope)?;
    super::webgl_shader_precision_format::install(scope)?;
    super::webgl_shader::install(scope)?;
    super::webgl_sampler::install(scope)?;
    super::webgl_rendering_context::install(scope)?;
    super::webgl_renderbuffer::install(scope)?;
    super::webgl_query::install(scope)?;
    super::webgl_program::install(scope)?;
    super::webgl_object::install(scope)?;
    super::webgl_framebuffer::install(scope)?;
    super::webgl_context_event::install(scope)?;
    super::webgl_buffer::install(scope)?;
    super::webgl_active_info::install(scope)?;
    super::webgl2_rendering_context::install(scope)?;
    super::restriction_target::install(scope)?;
    super::rtc_transform_event::install_in_worker_realm(scope)?;
    super::rtc_rtp_script_transformer::install_in_worker_realm(scope)?;
    super::rtc_data_channel::install(scope)?;
    super::rtc_encoded_video_frame::install(scope)?;
    super::rtc_encoded_audio_frame::install(scope)?;
    super::quota_exceeded_error::install(scope)?;
    super::push_subscription_options::install(scope)?;
    super::push_subscription::install(scope)?;
    super::push_manager::install(scope)?;
    super::periodic_sync_manager::install(scope)?;
    super::origin::install(scope)?;
    super::notification::install(scope)?;
    super::crop_target::install(scope)?;
    super::background_fetch_registration::install(scope)?;
    super::background_fetch_record::install(scope)?;
    super::background_fetch_manager::install(scope)?;
    super::xml_http_request_upload::install(scope)?;
    super::xml_http_request_event_target::install(scope)?;
    super::xml_http_request::install(scope)?;
    super::cache::install(scope)?;
    super::cache_storage::install(scope)?;
    super::subtle_crypto::install(scope)?;
    super::crypto_key::install(scope)?;
    super::crypto::install(scope)?;
    super::performance::install(scope)?;
    super::performance_entry::install(scope)?;
    super::performance_mark::install(scope)?;
    super::performance_measure::install(scope)?;
    super::performance_server_timing::install(scope)?;
    super::performance_resource_timing::install(scope)?;
    super::performance_observer_entry_list::install(scope)?;
    super::performance_observer::install(scope)?;
    super::offscreen_canvas::install(scope)?;
    super::offscreen_canvas_rendering_context_2d::install(scope)?;
    super::image_bitmap::install(scope)?;
    super::image_bitmap_rendering_context::install(scope)?;
    super::image_data::install(scope)?;
    super::path2d::install(scope)?;
    super::text_metrics::install(scope)?;
    super::dom_matrix::install_standard_name(scope)?;
    super::dom_matrix_read_only::install(scope)?;
    super::dom_point::install(scope)?;
    super::dom_point_read_only::install(scope)?;
    super::dom_quad::install(scope)?;
    super::dom_rect::install(scope)?;
    super::dom_rect_read_only::install(scope)?;
    super::dom_string_list::install(scope)?;
    super::video_frame::install(scope)?;
    super::video_color_space::install(scope)?;
    super::user_activation::install(scope)?;
    super::trusted_type_policy_factory::install(scope)?;
    super::trusted_type_policy::install(scope)?;
    super::trusted_script_url::install(scope)?;
    super::trusted_script::install(scope)?;
    super::trusted_html::install(scope)?;
    super::task_signal::install(scope)?;
    super::task_priority_change_event::install(scope)?;
    super::task_controller::install(scope)?;
    super::sync_manager::install(scope)?;
    super::subscriber::install(scope)?;
    super::source_buffer_list::install(scope)?;
    super::source_buffer::install(scope)?;
    super::security_policy_violation_event::install(scope)?;
    super::scheduler::install(scope)?;
    super::reporting_observer::install(scope)?;
    super::report_body::install(scope)?;
    super::progress_event::install(scope)?;
    super::permissions::install(scope)?;
    super::permission_status::install(scope)?;
    super::observable::install(scope)?;
    super::network_information::install(scope)?;
    super::navigator_ua_data::install(scope)?;
    super::media_source_handle::install(scope)?;
    super::media_source::install(scope)?;
    super::media_capabilities::install(scope)?;
    super::idb_version_change_event::install(scope)?;
    super::idb_transaction::install(scope)?;
    super::idb_request::install(scope)?;
    super::idb_record::install(scope)?;
    super::idb_open_db_request::install(scope)?;
    super::idb_object_store::install(scope)?;
    super::idb_key_range::install(scope)?;
    super::idb_index::install(scope)?;
    super::idb_factory::install(scope)?;
    super::idb_database::install(scope)?;
    super::idb_cursor_with_value::install(scope)?;
    super::idb_cursor::install(scope)?;
    super::font_face::install(scope)?;
    super::file_reader_sync::install_in_worker_realm(scope)?;
    super::file_list::install(scope)?;
    super::event_source::install(scope)?;
    super::encoded_video_chunk::install(scope)?;
    super::encoded_audio_chunk::install(scope)?;
    super::audio_data::install(scope)?;
    super::close_event::install(scope)?;
    super::canvas_pattern::install(scope)?;
    super::canvas_gradient::install(scope)?;
    super::css_skew_y::install(scope)?;
    super::css_skew_x::install(scope)?;
    super::audio_decoder::install(scope)?;
    super::audio_encoder::install(scope)?;
    super::create_monitor::install(scope)?;
    super::file_system_sync_access_handle::install_in_worker_realm(scope)?;
    super::gpu::install(scope)?;
    super::gpu_adapter::install(scope)?;
    super::gpu_adapter_info::install(scope)?;
    super::gpu_bind_group::install(scope)?;
    super::gpu_bind_group_layout::install(scope)?;
    super::gpu_buffer::install(scope)?;
    super::gpu_buffer_usage::install(scope)?;
    super::gpu_canvas_context::install(scope)?;
    super::gpu_color_write::install(scope)?;
    super::gpu_command_buffer::install(scope)?;
    super::gpu_command_encoder::install(scope)?;
    super::gpu_compilation_info::install(scope)?;
    super::gpu_compilation_message::install(scope)?;
    super::gpu_compute_pass_encoder::install(scope)?;
    super::gpu_compute_pipeline::install(scope)?;
    super::gpu_device::install(scope)?;
    super::gpu_device_lost_info::install(scope)?;
    super::gpu_error::install(scope)?;
    super::gpu_external_texture::install(scope)?;
    super::gpu_internal_error::install(scope)?;
    super::gpu_map_mode::install(scope)?;
    super::gpu_out_of_memory_error::install(scope)?;
    super::gpu_pipeline_error::install(scope)?;
    super::gpu_pipeline_layout::install(scope)?;
    super::gpu_query_set::install(scope)?;
    super::gpu_queue::install(scope)?;
    super::gpu_render_bundle::install(scope)?;
    super::gpu_render_bundle_encoder::install(scope)?;
    super::gpu_render_pass_encoder::install(scope)?;
    super::gpu_render_pipeline::install(scope)?;
    super::gpu_sampler::install(scope)?;
    super::gpu_shader_module::install(scope)?;
    super::gpu_shader_stage::install(scope)?;
    super::gpu_supported_features::install(scope)?;
    super::gpu_supported_limits::install(scope)?;
    super::gpu_texture::install(scope)?;
    super::gpu_texture_usage::install(scope)?;
    super::gpu_texture_view::install(scope)?;
    super::gpu_uncaptured_error_event::install(scope)?;
    super::gpu_validation_error::install(scope)?;
    super::idle_detector::install(scope)?;
    super::image_decoder::install(scope)?;
    super::image_track::install(scope)?;
    super::image_track_list::install(scope)?;
    super::navigation_preload_manager::install(scope)?;
    super::service_worker_registration::install(scope)?;
    super::storage_manager::install(scope)?;
    super::video_decoder::install(scope)?;
    super::video_encoder::install(scope)?;
    super::wgsl_language_features::install(scope)?;
    super::web_transport::install(scope)?;
    super::web_transport_bidirectional_stream::install(scope)?;
    super::web_transport_datagram_duplex_stream::install(scope)?;
    super::web_transport_error::install(scope)?;
    super::file_system_handle::install(scope)?;
    super::file_system_directory_handle::install(scope)?;
    super::file_system_file_handle::install_in_worker_realm(scope)?;
    super::file_system_writable_file_stream::install(scope)?;
    super::file_system_observer::install(scope)?;
    super::hid::install(scope)?;
    super::hid_connection_event::install(scope)?;
    super::hid_device::install(scope)?;
    super::hid_input_report_event::install(scope)?;
    super::lock::install(scope)?;
    super::lock_manager::install(scope)?;
    super::pressure_observer::install(scope)?;
    super::pressure_record::install(scope)?;
    super::serial::install(scope)?;
    super::serial_port::install(scope)?;
    super::storage_bucket::install(scope)?;
    super::storage_bucket_manager::install(scope)?;
    super::usb::install(scope)?;
    super::usb_alternate_interface::install(scope)?;
    super::usb_configuration::install(scope)?;
    super::usb_connection_event::install(scope)?;
    super::usb_device::install(scope)?;
    super::usb_endpoint::install(scope)?;
    super::usb_in_transfer_result::install(scope)?;
    super::usb_interface::install(scope)?;
    super::usb_isochronous_in_transfer_packet::install(scope)?;
    super::usb_isochronous_in_transfer_result::install(scope)?;
    super::usb_isochronous_out_transfer_packet::install(scope)?;
    super::usb_isochronous_out_transfer_result::install(scope)?;
    super::usb_out_transfer_result::install(scope)?;
    super::console_edge::install(scope)?;
    let shared_array_buffer = crate::webidl::string(scope, "SharedArrayBuffer")?;
    let _ = scope
        .get_current_context()
        .global(scope)
        .delete(scope, shared_array_buffer.into());
    Ok(())
}

pub(crate) fn mirror_global_to_target(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, name)?;
    let value = global
        .get(scope, key.into())
        .ok_or_else(|| format!("Worker global {name} is unavailable"))?;
    match target.define_own_property(scope, key.into(), value, v8::PropertyAttribute::DONT_ENUM) {
        Some(true) => Ok(()),
        _ => Err(format!("cannot mirror Worker global {name}")),
    }
}

fn mirror_worker_intrinsics_to_target(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    kind: RealmKind,
) -> Result<(), String> {
    let constructor = crate::webidl::string(scope, "constructor")?;
    let _ = target.delete(scope, constructor.into());
    mirror_global_to_target(scope, target, "Object")?;
    mirror_global_to_target(scope, target, "Function")?;
    mirror_global_to_target(scope, target, "Array")?;
    mirror_global_to_target(scope, target, "Number")?;
    mirror_global_to_target(scope, target, "parseFloat")?;
    mirror_global_to_target(scope, target, "parseInt")?;
    mirror_global_to_target(scope, target, "Infinity")?;
    mirror_global_to_target(scope, target, "NaN")?;
    mirror_global_to_target(scope, target, "undefined")?;
    mirror_global_to_target(scope, target, "Boolean")?;
    mirror_global_to_target(scope, target, "String")?;
    mirror_global_to_target(scope, target, "Symbol")?;
    mirror_global_to_target(scope, target, "Date")?;
    mirror_global_to_target(scope, target, "Promise")?;
    mirror_global_to_target(scope, target, "RegExp")?;
    mirror_global_to_target(scope, target, "Error")?;
    mirror_global_to_target(scope, target, "AggregateError")?;
    mirror_global_to_target(scope, target, "EvalError")?;
    mirror_global_to_target(scope, target, "RangeError")?;
    mirror_global_to_target(scope, target, "ReferenceError")?;
    mirror_global_to_target(scope, target, "SyntaxError")?;
    mirror_global_to_target(scope, target, "TypeError")?;
    mirror_global_to_target(scope, target, "URIError")?;
    let global_this = crate::webidl::string(scope, "globalThis")?;
    if target.define_own_property(
        scope,
        global_this.into(),
        target.into(),
        v8::PropertyAttribute::NONE,
    ) != Some(true)
    {
        return Err("cannot define Worker globalThis on global target".to_owned());
    }
    crate::webidl::define_global(scope, "globalThis", target.into())?;
    mirror_global_to_target(scope, target, "JSON")?;
    mirror_global_to_target(scope, target, "Math")?;
    mirror_global_to_target(scope, target, "Intl")?;
    mirror_global_to_target(scope, target, "ArrayBuffer")?;
    mirror_global_to_target(scope, target, "Atomics")?;
    mirror_global_to_target(scope, target, "Uint8Array")?;
    mirror_global_to_target(scope, target, "Int8Array")?;
    mirror_global_to_target(scope, target, "Uint16Array")?;
    mirror_global_to_target(scope, target, "Int16Array")?;
    mirror_global_to_target(scope, target, "Uint32Array")?;
    mirror_global_to_target(scope, target, "Int32Array")?;
    mirror_global_to_target(scope, target, "BigUint64Array")?;
    mirror_global_to_target(scope, target, "BigInt64Array")?;
    mirror_global_to_target(scope, target, "Uint8ClampedArray")?;
    mirror_global_to_target(scope, target, "Float32Array")?;
    mirror_global_to_target(scope, target, "Float64Array")?;
    mirror_global_to_target(scope, target, "DataView")?;
    mirror_global_to_target(scope, target, "Map")?;
    mirror_global_to_target(scope, target, "BigInt")?;
    mirror_global_to_target(scope, target, "Set")?;
    mirror_global_to_target(scope, target, "Iterator")?;
    mirror_global_to_target(scope, target, "WeakMap")?;
    mirror_global_to_target(scope, target, "WeakSet")?;
    mirror_global_to_target(scope, target, "Proxy")?;
    mirror_global_to_target(scope, target, "Reflect")?;
    mirror_global_to_target(scope, target, "FinalizationRegistry")?;
    mirror_global_to_target(scope, target, "WeakRef")?;
    mirror_global_to_target(scope, target, "decodeURI")?;
    mirror_global_to_target(scope, target, "decodeURIComponent")?;
    mirror_global_to_target(scope, target, "encodeURI")?;
    mirror_global_to_target(scope, target, "encodeURIComponent")?;
    mirror_global_to_target(scope, target, "escape")?;
    mirror_global_to_target(scope, target, "unescape")?;
    mirror_global_to_target(scope, target, "eval")?;
    mirror_global_to_target(scope, target, "isFinite")?;
    mirror_global_to_target(scope, target, "isNaN")?;
    if kind != RealmKind::Dedicated {
        mirror_global_to_target(scope, target, "Temporal")?;
        mirror_global_to_target(scope, target, "SuppressedError")?;
        mirror_global_to_target(scope, target, "DisposableStack")?;
        mirror_global_to_target(scope, target, "AsyncDisposableStack")?;
        mirror_global_to_target(scope, target, "Float16Array")?;
        mirror_global_to_target(scope, target, "WebAssembly")?;
    }
    Ok(())
}

fn mirror_realm_local_globals(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    mirror_global_to_target(scope, target, "EventTarget")?;
    mirror_global_to_target(scope, target, "Event")?;
    mirror_global_to_target(scope, target, "CustomEvent")?;
    mirror_global_to_target(scope, target, "MessageEvent")?;
    mirror_global_to_target(scope, target, "ErrorEvent")?;
    mirror_global_to_target(scope, target, "PromiseRejectionEvent")?;
    mirror_global_to_target(scope, target, "DOMException")?;
    mirror_global_to_target(scope, target, "AbortSignal")?;
    mirror_global_to_target(scope, target, "AbortController")?;
    mirror_global_to_target(scope, target, "MessagePort")?;
    mirror_global_to_target(scope, target, "MessageChannel")?;
    mirror_global_to_target(scope, target, "BroadcastChannel")?;
    mirror_global_to_target(scope, target, "URL")?;
    mirror_global_to_target(scope, target, "URLSearchParams")?;
    mirror_global_to_target(scope, target, "URLPattern")?;
    mirror_global_to_target(scope, target, "Blob")?;
    mirror_global_to_target(scope, target, "File")?;
    mirror_global_to_target(scope, target, "FileReader")?;
    mirror_global_to_target(scope, target, "Headers")?;
    mirror_global_to_target(scope, target, "Request")?;
    mirror_global_to_target(scope, target, "Response")?;
    mirror_global_to_target(scope, target, "FormData")?;
    mirror_global_to_target(scope, target, "Cache")?;
    mirror_global_to_target(scope, target, "CacheStorage")?;
    mirror_global_to_target(scope, target, "Crypto")?;
    mirror_global_to_target(scope, target, "SubtleCrypto")?;
    mirror_global_to_target(scope, target, "CryptoKey")?;
    mirror_global_to_target(scope, target, "Performance")?;
    mirror_global_to_target(scope, target, "OffscreenCanvas")?;
    mirror_global_to_target(scope, target, "OffscreenCanvasRenderingContext2D")?;
    mirror_global_to_target(scope, target, "WebGLRenderingContext")?;
    mirror_global_to_target(scope, target, "WebGL2RenderingContext")?;
    mirror_global_to_target(scope, target, "WebGLContextEvent")?;
    mirror_global_to_target(scope, target, "CloseEvent")?;
    mirror_global_to_target(scope, target, "ProgressEvent")?;
    mirror_global_to_target(scope, target, "SecurityPolicyViolationEvent")?;
    mirror_global_to_target(scope, target, "TaskPriorityChangeEvent")?;
    mirror_global_to_target(scope, target, "TrustedTypePolicyFactory")?;
    mirror_global_to_target(scope, target, "Scheduler")?;
    mirror_global_to_target(scope, target, "IDBFactory")?;
    mirror_global_to_target(scope, target, "console")
}

fn global_function<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, name)?;
    let value = global
        .get(scope, key.into())
        .ok_or_else(|| format!("{name} is unavailable in Worker realm"))?;
    v8::Local::<v8::Function>::try_from(value)
        .map_err(|_| format!("{name} is not a constructor in Worker realm"))
}

fn illegal_worker_scope_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn illegal_dedicated_scope_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn illegal_shared_scope_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn illegal_service_scope_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn current_realm_id(scope: &v8::PinScope<'_, '_>) -> Option<i32> {
    let global_id = scope
        .get_current_context()
        .global(scope)
        .get_identity_hash()
        .get();
    scope
        .get_slot::<WorkerRealmStore>()?
        .records
        .iter()
        .find_map(|(realm_id, record)| {
            (v8::Local::new(scope, &record.global)
                .get_identity_hash()
                .get()
                == global_id)
                .then_some(*realm_id)
        })
}

pub(crate) fn record(scope: &v8::PinScope<'_, '_>, realm_id: i32) -> Option<WorkerRealmRecord> {
    scope
        .get_slot::<WorkerRealmStore>()?
        .records
        .get(&realm_id)
        .cloned()
}

pub(crate) fn module_url(
    scope: &v8::PinScope<'_, '_>,
    module: v8::Local<'_, v8::Module>,
) -> Option<String> {
    let script_id = module.script_id()?;
    scope
        .get_slot::<WorkerRealmStore>()?
        .records
        .values()
        .find_map(|record| record.module_urls.get(&script_id).cloned())
}

pub(crate) fn current_record(scope: &v8::PinScope<'_, '_>) -> Option<WorkerRealmRecord> {
    record(scope, current_realm_id(scope)?)
}

pub(crate) fn get_self(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(global) = current_record(scope).and_then(|record| record.global_target) {
        result.set(v8::Local::new(scope, &global).into());
    }
}

pub(crate) fn get_location(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(location) = current_record(scope).and_then(|record| record.location) {
        result.set(v8::Local::new(scope, &location).into());
    }
}

pub(crate) fn get_navigator(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(navigator) = current_record(scope).and_then(|record| record.navigator) {
        result.set(v8::Local::new(scope, &navigator).into());
    }
}

pub(crate) fn get_crypto(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(crypto) = current_record(scope).and_then(|record| record.crypto) {
        result.set(v8::Local::new(scope, &crypto).into());
    }
}

pub(crate) fn get_performance(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(performance) = current_record(scope).and_then(|record| record.performance) {
        result.set(v8::Local::new(scope, &performance).into());
    }
}

pub(crate) fn get_scheduler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(scheduler) = current_record(scope).and_then(|record| record.scheduler) {
        result.set(v8::Local::new(scope, &scheduler).into());
    }
}

pub(crate) fn get_trusted_types(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(trusted_types) = current_record(scope).and_then(|record| record.trusted_types) {
        result.set(v8::Local::new(scope, &trusted_types).into());
    }
}

pub(crate) fn get_caches(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(caches) = current_record(scope).and_then(|record| record.caches) {
        result.set(v8::Local::new(scope, &caches).into());
    }
}

pub(crate) fn get_indexed_db(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(indexed_db) = current_record(scope).and_then(|record| record.indexed_db) {
        result.set(v8::Local::new(scope, &indexed_db).into());
    }
}

pub(crate) fn get_fonts(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(fonts) = current_record(scope).and_then(|record| record.fonts) {
        result.set(v8::Local::new(scope, &fonts).into());
    }
}

pub(crate) fn get_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(origin) = current_record(scope).map(|record| record.origin) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &origin) {
        result.set(value.into());
    }
}

pub(crate) fn current_origin(scope: &v8::PinScope<'_, '_>) -> Option<String> {
    current_record(scope).map(|record| record.origin)
}

pub(crate) fn get_worker_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    value: bool,
    mut result: v8::ReturnValue<'_>,
) {
    if valid_global_receiver(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_replaceable_global(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
) {
    if !valid_global_receiver(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let _ = arguments.this().define_own_property(
        scope,
        key.into(),
        arguments.get(0),
        v8::PropertyAttribute::NONE,
    );
}

fn valid_global_receiver(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    if object.strict_equals(scope.get_current_context().global(scope).into()) {
        return true;
    }
    current_record(scope).is_some_and(|record| {
        object.strict_equals(v8::Local::new(scope, &record.global).into())
            || record
                .global_target
                .is_some_and(|target| object.strict_equals(v8::Local::new(scope, &target).into()))
    })
}

#[derive(Clone, Copy)]
pub(crate) enum HandlerKind {
    Error,
    LanguageChange,
    Offline,
    Online,
    UnhandledRejection,
    RejectionHandled,
    Message,
    MessageError,
    Connect,
    RtcTransform,
}

pub(crate) fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    kind: HandlerKind,
    mut result: v8::ReturnValue<'_>,
) {
    let handler = current_record(scope).and_then(|record| match kind {
        HandlerKind::Error => record.onerror,
        HandlerKind::LanguageChange => record.onlanguagechange,
        HandlerKind::Offline => record.onoffline,
        HandlerKind::Online => record.ononline,
        HandlerKind::UnhandledRejection => record.onunhandledrejection,
        HandlerKind::RejectionHandled => record.onrejectionhandled,
        HandlerKind::Message => record.onmessage,
        HandlerKind::MessageError => record.onmessageerror,
        HandlerKind::Connect => record.onconnect,
        HandlerKind::RtcTransform => record.onrtctransform,
    });
    if let Some(handler) = handler {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    kind: HandlerKind,
    value: v8::Local<'_, v8::Value>,
) {
    let handler = v8::Local::<v8::Function>::try_from(value)
        .ok()
        .map(|function| v8::Global::new(scope, v8::Local::<v8::Value>::from(function)));
    let Some(realm_id) = current_realm_id(scope) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match kind {
        HandlerKind::Error => record.onerror = handler,
        HandlerKind::LanguageChange => record.onlanguagechange = handler,
        HandlerKind::Offline => record.onoffline = handler,
        HandlerKind::Online => record.ononline = handler,
        HandlerKind::UnhandledRejection => record.onunhandledrejection = handler,
        HandlerKind::RejectionHandled => record.onrejectionhandled = handler,
        HandlerKind::Message => record.onmessage = handler,
        HandlerKind::MessageError => record.onmessageerror = handler,
        HandlerKind::Connect => record.onconnect = handler,
        HandlerKind::RtcTransform => record.onrtctransform = handler,
    }
}

pub(crate) fn get_service_handler(
    scope: &mut v8::PinScope<'_, '_>,
    kind: ServiceHandlerKind,
    mut result: v8::ReturnValue<'_>,
) {
    let handler =
        current_record(scope).and_then(|record| record.service_handlers.get(&kind).cloned());
    if let Some(handler) = handler {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_service_handler(
    scope: &mut v8::PinScope<'_, '_>,
    kind: ServiceHandlerKind,
    value: v8::Local<'_, v8::Value>,
) {
    let Some(realm_id) = current_realm_id(scope) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let handler = v8::Local::<v8::Function>::try_from(value)
        .ok()
        .map(|function| v8::Global::new(scope, v8::Local::<v8::Value>::from(function)));
    let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.kind != RealmKind::Service {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(handler) = handler {
        record.service_handlers.insert(kind, handler);
    } else {
        record.service_handlers.remove(&kind);
    }
}

pub(crate) fn bind_service_objects(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    registration: v8::Local<'_, v8::Object>,
    worker: v8::Local<'_, v8::Object>,
    container_id: i32,
) -> Result<(), String> {
    let record = record(scope, realm_id)
        .filter(|record| record.kind == RealmKind::Service)
        .ok_or_else(|| "Service worker realm is missing".to_owned())?;
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let clients = super::service_worker_clients::create(worker_scope, container_id)?;
    let worker = super::service_worker::create_alias(worker_scope, worker)?;
    let registration =
        super::service_worker_registration::create_alias(worker_scope, registration, worker)?;
    let registration = v8::Global::new(worker_scope, registration);
    let worker = v8::Global::new(worker_scope, worker);
    let clients = v8::Global::new(worker_scope, clients);
    let Some(record) = worker_scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    else {
        return Err("Service worker realm disappeared".to_owned());
    };
    record.service_registration = Some(registration);
    record.service_worker = Some(worker);
    record.service_clients = Some(clients);
    Ok(())
}

pub(crate) fn get_service_object(
    scope: &mut v8::PinScope<'_, '_>,
    property: ServiceObjectKind,
    mut result: v8::ReturnValue<'_>,
) {
    let value = current_record(scope).and_then(|record| match property {
        ServiceObjectKind::Clients => record.service_clients,
        ServiceObjectKind::Registration => record.service_registration,
        ServiceObjectKind::ServiceWorker => record.service_worker,
    });
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

#[derive(Clone, Copy)]
pub(crate) enum ServiceObjectKind {
    Clients,
    Registration,
    ServiceWorker,
}

pub(crate) fn skip_waiting(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if current_record(scope).is_none_or(|record| record.kind != RealmKind::Service) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(promise) =
        super::writable_stream::resolved_promise(scope, v8::undefined(scope).into())
    {
        result.set(promise.into());
    }
}

pub(crate) fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = current_record(scope)
        && let Some(value) = v8::String::new(scope, &record.name)
    {
        result.set(value.into());
    }
}

pub(crate) fn post_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'postMessage': 1 argument required",
        );
        return;
    }
    let Some(realm_id) = current_realm_id(scope) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current_record(scope)
        .is_none_or(|record| record.kind != RealmKind::Dedicated || record.closed)
    {
        return;
    }
    let Ok(message) =
        super::worker_structured_clone::serialize(scope, arguments.get(0), arguments.get(1))
    else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    {
        record.outgoing.push(message);
    }
}

pub(crate) fn close(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(realm_id) = current_realm_id(scope) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    mark_closed(scope, realm_id);
}

pub(crate) fn mark_closed(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    {
        record.closed = true;
        record.timeouts.clear();
        record.intervals.clear();
        record.animation_frames.clear();
        record.animation_frame_due_ms = None;
    }
}

pub(crate) fn terminate_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    mark_closed(scope, realm_id);
    if let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    {
        record.outgoing.clear();
    }
    reclaim_realm(scope, realm_id);
}

pub(crate) fn is_closed(scope: &v8::PinScope<'_, '_>, realm_id: i32) -> bool {
    record(scope, realm_id).is_none_or(|record| record.closed)
}

pub(crate) fn reclaim_closed_realms(scope: &mut v8::PinScope<'_, '_>) {
    let closed = scope
        .get_slot::<WorkerRealmStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter_map(|(realm_id, record)| record.closed.then_some(*realm_id))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for realm_id in closed {
        reclaim_realm(scope, realm_id);
    }
}

fn reclaim_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    let context = scope
        .get_slot::<WorkerRealmStore>()
        .and_then(|store| store.records.get(&realm_id))
        .map(|record| record.context.clone());
    let Some(context) = context else {
        return;
    };
    super::worker::terminate_children_for_parent_context(scope, &context);
    super::worker_realm_interfaces::cleanup(scope, realm_id);
    if let Some(store) = scope.get_slot_mut::<WorkerRealmStore>() {
        store.records.remove(&realm_id);
    }
}

pub(crate) fn import_scripts(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = current_record(scope) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.module {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'importScripts': Module scripts do not support importScripts",
        );
        return;
    }
    let _user_execution = crate::trace::enter_user_execution(scope);
    for index in 0..arguments.length() {
        let input = crate::webidl::value_to_string(scope, arguments.get(index));
        let script = match super::worker_script_source::load(scope, &input, Some(&record.url)) {
            Ok(script) => script,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        let Some(source) = v8::String::new(scope, &script.source) else {
            crate::webidl::throw_type_error(scope, "Imported worker script exceeds V8 limits");
            return;
        };
        let Some(compiled) = v8::Script::compile(scope, source, None) else {
            return;
        };
        if compiled.run(scope).is_none() {
            return;
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WorkerScriptError {
    pub(crate) message: String,
    pub(crate) filename: String,
    pub(crate) lineno: u32,
    pub(crate) colno: u32,
}

impl WorkerScriptError {
    fn fallback(message: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            filename: filename.into(),
            lineno: 0,
            colno: 0,
        }
    }
}

pub(crate) fn evaluate(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    source: &str,
) -> Result<(), WorkerScriptError> {
    let record = record(scope, realm_id)
        .ok_or_else(|| WorkerScriptError::fallback("Worker realm is missing", ""))?;
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    v8::tc_scope!(let try_catch, worker_scope);
    let _user_execution = crate::trace::enter_user_execution(try_catch);
    let source = v8::String::new(try_catch, source).ok_or_else(|| {
        WorkerScriptError::fallback("Worker source exceeds V8 limits", &record.url)
    })?;
    let success = if record.module {
        compile_module(try_catch, realm_id, &record.url, source).and_then(|module| {
            module
                .instantiate_module(try_catch, resolve_module)
                .and_then(|instantiated| instantiated.then(|| module))
                .and_then(|module| module.evaluate(try_catch))
        })
    } else {
        v8::String::new(try_catch, &record.url).and_then(|resource_name| {
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
            v8::Script::compile(try_catch, source, Some(&origin))
                .and_then(|script| script.run(try_catch))
        })
    };
    if success.is_none() {
        return Err(exception_detail(
            try_catch,
            "Worker script execution failed",
            &record.url,
        ));
    }
    try_catch.perform_microtask_checkpoint();
    Ok(())
}

fn resolve_module<'s>(
    context: v8::Local<'s, v8::Context>,
    specifier: v8::Local<'s, v8::String>,
    _import_attributes: v8::Local<'s, v8::FixedArray>,
    referrer: v8::Local<'s, v8::Module>,
) -> Option<v8::Local<'s, v8::Module>> {
    v8::callback_scope!(unsafe scope, context);
    let realm_id = current_realm_id(scope)?;
    let base = record(scope, realm_id).and_then(|record| {
        referrer
            .script_id()
            .and_then(|script_id| record.module_urls.get(&script_id).cloned())
            .or(Some(record.url))
    })?;
    let input = specifier.to_rust_string_lossy(scope);
    let script = match super::worker_script_source::load(scope, &input, Some(&base)) {
        Ok(script) => script,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    if let Some(module) = scope
        .get_slot::<WorkerRealmStore>()
        .and_then(|store| store.records.get(&realm_id))
        .and_then(|record| record.modules.get(&script.url))
        .cloned()
    {
        return Some(v8::Local::new(scope, &module));
    }
    let source = v8::String::new(scope, &script.source)?;
    compile_module(scope, realm_id, &script.url, source)
}

fn compile_module<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    realm_id: i32,
    url: &str,
    source: v8::Local<'s, v8::String>,
) -> Option<v8::Local<'s, v8::Module>> {
    let resource_name = v8::String::new(scope, url)?;
    let origin = v8::ScriptOrigin::new(
        scope,
        resource_name.into(),
        0,
        0,
        false,
        -1,
        None,
        false,
        false,
        true,
        None,
    );
    let mut source = v8::script_compiler::Source::new(source, Some(&origin));
    let module = v8::script_compiler::compile_module(scope, &mut source)?;
    let saved_module = v8::Global::new(scope, module);
    if let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    {
        if let Some(script_id) = module.script_id() {
            record.module_urls.insert(script_id, url.to_owned());
        }
        record.modules.insert(url.to_owned(), saved_module);
    }
    Some(module)
}

fn dynamic_import<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    _host_defined_options: v8::Local<'s, v8::Data>,
    resource_name: v8::Local<'s, v8::Value>,
    specifier: v8::Local<'s, v8::String>,
    _import_attributes: v8::Local<'s, v8::FixedArray>,
) -> Option<v8::Local<'s, v8::Promise>> {
    if super::worklet::current_worklet_id(scope).is_some() {
        return super::worklet::dynamic_import(scope, specifier);
    }
    if current_realm_id(scope).is_none() {
        return super::html_script_element::dynamic_import(scope, resource_name, specifier);
    }
    let realm_id = current_realm_id(scope)?;
    let base = record(scope, realm_id)?.url;
    let input = specifier.to_rust_string_lossy(scope);
    let script = match super::worker_script_source::load(scope, &input, Some(&base)) {
        Ok(script) => script,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    let source = v8::String::new(scope, &script.source)?;
    let module = compile_module(scope, realm_id, &script.url, source)?;
    if module.instantiate_module(scope, resolve_module) != Some(true) {
        return None;
    }
    module.evaluate(scope)?;
    let resolver = v8::PromiseResolver::new(scope)?;
    if resolver.resolve(scope, module.get_module_namespace()) != Some(true) {
        return None;
    }
    Some(resolver.get_promise(scope))
}

fn exception_detail(
    scope: &v8::PinnedRef<'_, v8::TryCatch<'_, '_, v8::HandleScope<'_, v8::Context>>>,
    fallback: &str,
    fallback_url: &str,
) -> WorkerScriptError {
    let message = scope
        .exception()
        .and_then(|exception| exception.to_string(scope))
        .map(|text| text.to_rust_string_lossy(scope))
        .unwrap_or_else(|| fallback.to_owned());
    let Some(detail) = scope.message() else {
        return WorkerScriptError::fallback(message, fallback_url);
    };
    let filename = detail
        .get_script_resource_name(scope)
        .and_then(|value| value.to_string(scope))
        .map(|value| value.to_rust_string_lossy(scope))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback_url.to_owned());
    WorkerScriptError {
        message,
        filename,
        lineno: detail.get_line_number(scope).unwrap_or_default() as u32,
        colno: detail.get_start_column().saturating_add(1) as u32,
    }
}

pub(crate) fn deliver_message(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    message: &super::worker_structured_clone::SerializedMessage,
) -> bool {
    let Some(record) = record(scope, realm_id).filter(|record| !record.closed) else {
        return false;
    };
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let Some(data) = super::worker_structured_clone::deserialize(worker_scope, message) else {
        let handler = record.onmessageerror.clone();
        dispatch_simple_event(worker_scope, &record, "messageerror", handler);
        return false;
    };
    let ports = message
        .ports
        .iter()
        .map(|port| v8::Local::new(worker_scope, port))
        .collect();
    let Ok(event) = super::message_event::create(worker_scope, "message", data, "", None, ports)
    else {
        return false;
    };
    let handler = record.onmessage.clone();
    dispatch_event(worker_scope, &record, event, "message", handler);
    worker_scope.perform_microtask_checkpoint();
    true
}

pub(crate) fn deliver_service_message(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    container_id: i32,
    message: &super::worker_structured_clone::SerializedMessage,
) -> bool {
    let Some(record) = record(scope, realm_id)
        .filter(|record| record.kind == RealmKind::Service && !record.closed)
    else {
        return false;
    };
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let Some(data) = super::worker_structured_clone::deserialize(worker_scope, message) else {
        dispatch_service_simple_event(
            worker_scope,
            &record,
            "messageerror",
            ServiceHandlerKind::MessageError,
        );
        return false;
    };
    let source = super::service_worker_clients::client_for_container(worker_scope, container_id)
        .map(v8::Local::<v8::Value>::from);
    let ports = message
        .ports
        .iter()
        .map(|port| v8::Local::new(worker_scope, port))
        .collect();
    let Ok(event) = super::message_event::create(worker_scope, "message", data, "", source, ports)
    else {
        return false;
    };
    let handler = record
        .service_handlers
        .get(&ServiceHandlerKind::Message)
        .cloned();
    dispatch_event(worker_scope, &record, event, "message", handler);
    worker_scope.perform_microtask_checkpoint();
    true
}

pub(crate) fn dispatch_service_lifecycle(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    event_type: &str,
    kind: ServiceHandlerKind,
) -> bool {
    let Some(record) = record(scope, realm_id)
        .filter(|record| record.kind == RealmKind::Service && !record.closed)
    else {
        return false;
    };
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    dispatch_service_simple_event(worker_scope, &record, event_type, kind);
    worker_scope.perform_microtask_checkpoint();
    true
}

pub(crate) fn dispatch_service_fetch(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    url: &str,
) -> Option<Result<v8::Global<v8::Value>, String>> {
    let record = record(scope, realm_id)
        .filter(|record| record.kind == RealmKind::Service && !record.closed)?;
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let url = v8::String::new(worker_scope, url)?;
    let request = match super::request::create_from_input(worker_scope, url.into()) {
        Ok(request) => request,
        Err(message) => return Some(Err(message)),
    };
    let event = match super::fetch_event::create(worker_scope, request, "") {
        Ok(event) => event,
        Err(message) => return Some(Err(message)),
    };
    let handler = record
        .service_handlers
        .get(&ServiceHandlerKind::Fetch)
        .cloned();
    dispatch_event(worker_scope, &record, event, "fetch", handler);
    worker_scope.perform_microtask_checkpoint();
    let response = super::fetch_event::take_response(worker_scope, event)?;
    let response = v8::Local::new(worker_scope, &response);
    if let Ok(promise) = v8::Local::<v8::Promise>::try_from(response) {
        worker_scope.perform_microtask_checkpoint();
        return Some(match promise.state() {
            v8::PromiseState::Fulfilled => {
                Ok(v8::Global::new(worker_scope, promise.result(worker_scope)))
            }
            v8::PromiseState::Rejected => Err(crate::webidl::value_to_string(
                worker_scope,
                promise.result(worker_scope),
            )),
            v8::PromiseState::Pending => {
                Err("ServiceWorker respondWith promise is still pending".to_owned())
            }
        });
    }
    Some(Ok(v8::Global::new(worker_scope, response)))
}

pub(crate) fn is_entered_service_realm(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let entered_id = scope
        .get_entered_or_microtask_context()
        .global(scope)
        .get_identity_hash()
        .get();
    let contexts = scope
        .get_slot::<WorkerRealmStore>()
        .map(|store| {
            store
                .records
                .values()
                .filter(|record| record.kind == RealmKind::Service)
                .map(|record| record.context.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    contexts.into_iter().any(|context| {
        v8::Local::new(scope, &context)
            .global(scope)
            .get_identity_hash()
            .get()
            == entered_id
    })
}

fn dispatch_service_simple_event(
    scope: &mut v8::PinScope<'_, '_>,
    record: &WorkerRealmRecord,
    event_type: &str,
    kind: ServiceHandlerKind,
) {
    let event = match super::extendable_event::create(scope, event_type) {
        Ok(event) => event,
        Err(_) => super::event_target::create_event(scope, event_type),
    };
    let handler = record.service_handlers.get(&kind).cloned();
    dispatch_event(scope, record, event, event_type, handler);
}

pub(crate) fn dispatch_connect(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    port: v8::Global<v8::Object>,
) -> bool {
    let Some(record) = record(scope, realm_id).filter(|record| !record.closed) else {
        return false;
    };
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let port = v8::Local::new(worker_scope, &port);
    let data = v8::null(worker_scope).into();
    let Ok(event) =
        super::message_event::create(worker_scope, "connect", data, "", None, vec![port])
    else {
        return false;
    };
    let handler = record.onconnect.clone();
    dispatch_event(worker_scope, &record, event, "connect", handler);
    worker_scope.perform_microtask_checkpoint();
    true
}

pub(crate) fn dispatch_rtc_transform(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    options: v8::Local<'_, v8::Value>,
) -> bool {
    let Some(record) = record(scope, realm_id)
        .filter(|record| record.kind == RealmKind::Dedicated && !record.closed)
    else {
        return false;
    };
    let options = v8::Global::new(scope, options);
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let options = v8::Local::new(worker_scope, &options);
    let Ok(transformer) = super::rtc_rtp_script_transformer::create(worker_scope, options) else {
        return false;
    };
    let Ok(event) = super::rtc_transform_event::create(worker_scope, transformer) else {
        return false;
    };
    dispatch_event(
        worker_scope,
        &record,
        event,
        "rtctransform",
        record.onrtctransform.clone(),
    );
    worker_scope.perform_microtask_checkpoint();
    true
}

fn dispatch_simple_event(
    scope: &mut v8::PinScope<'_, '_>,
    record: &WorkerRealmRecord,
    event_type: &str,
    handler: Option<v8::Global<v8::Value>>,
) {
    let event = super::event_target::create_event(scope, event_type);
    dispatch_event(scope, record, event, event_type, handler);
}

fn dispatch_event(
    scope: &mut v8::PinScope<'_, '_>,
    record: &WorkerRealmRecord,
    event: v8::Local<'_, v8::Object>,
    _event_type: &str,
    handler: Option<v8::Global<v8::Value>>,
) {
    let _user_execution = crate::trace::enter_user_execution(scope);
    let target = v8::Local::new(scope, &record.global);
    super::event_target::dispatch(scope, target, event);
    if let Some(handler) = handler
        && let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler))
    {
        let _ = function.call(scope, target.into(), &[event.into()]);
    }
}

pub(crate) fn dispatch_error(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    message: &str,
) -> bool {
    let filename = record(scope, realm_id)
        .map(|record| record.url)
        .unwrap_or_default();
    dispatch_script_error(
        scope,
        realm_id,
        &WorkerScriptError::fallback(message, filename),
    )
}

pub(crate) fn dispatch_script_error(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    detail: &WorkerScriptError,
) -> bool {
    let Some(record) = record(scope, realm_id).filter(|record| !record.closed) else {
        return false;
    };
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let error = v8::Exception::error(
        worker_scope,
        v8::String::new(worker_scope, &detail.message).expect("short Worker error"),
    );
    let Ok(event) = super::error_event::create_detailed(
        worker_scope,
        "error",
        detail.message.clone(),
        detail.filename.clone(),
        detail.lineno,
        detail.colno,
        error,
    ) else {
        return false;
    };
    let target = v8::Local::new(worker_scope, &record.global);
    super::event_target::dispatch(worker_scope, target, event);
    let Some(handler) = record.onerror else {
        return false;
    };
    let Ok(function) = v8::Local::<v8::Function>::try_from(v8::Local::new(worker_scope, &handler))
    else {
        return false;
    };
    function
        .call(
            worker_scope,
            target.into(),
            &[
                v8::String::new(worker_scope, &detail.message)
                    .expect("short Worker error")
                    .into(),
                v8::String::new(worker_scope, &detail.filename)
                    .expect("Worker URL")
                    .into(),
                v8::Integer::new_from_unsigned(worker_scope, detail.lineno).into(),
                v8::Integer::new_from_unsigned(worker_scope, detail.colno).into(),
                error,
            ],
        )
        .is_some_and(|value| value.boolean_value(worker_scope))
}

pub(crate) fn take_outgoing(
    scope: &mut v8::PinScope<'_, '_>,
) -> Vec<(
    RealmOwner,
    super::worker_structured_clone::SerializedMessage,
)> {
    let Some(store) = scope.get_slot_mut::<WorkerRealmStore>() else {
        return Vec::new();
    };
    let mut outgoing = Vec::new();
    for record in store.records.values_mut() {
        let owner = record.owner;
        outgoing.extend(
            std::mem::take(&mut record.outgoing)
                .into_iter()
                .map(|message| (owner, message)),
        );
    }
    outgoing
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<WorkerRealmStore>().and_then(|store| {
        store
            .records
            .values()
            .filter(|record| !record.closed)
            .flat_map(|record| {
                record
                    .timeouts
                    .values()
                    .map(|timer| timer.due_ms)
                    .chain(record.intervals.values().map(|timer| timer.due_ms))
                    .chain(record.animation_frame_due_ms)
            })
            .min_by(f64::total_cmp)
    })
}

pub(crate) fn run_timers(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let now = crate::determinism::elapsed_milliseconds(scope);
    let mut ready = scope
        .get_slot::<WorkerRealmStore>()
        .map(|store| {
            let mut timers = Vec::new();
            for (realm_id, record) in &store.records {
                if record.closed {
                    continue;
                }
                timers.extend(
                    record
                        .timeouts
                        .iter()
                        .filter(|(_, timer)| timer.due_ms <= now)
                        .map(|(id, timer)| (timer.due_ms, *realm_id, *id, false)),
                );
                timers.extend(
                    record
                        .intervals
                        .iter()
                        .filter(|(_, timer)| timer.due_ms <= now)
                        .map(|(id, timer)| (timer.due_ms, *realm_id, *id, true)),
                );
            }
            timers
        })
        .unwrap_or_default();
    ready.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut ran = false;
    for (_, realm_id, id, repeating) in ready {
        let timer = scope
            .get_slot_mut::<WorkerRealmStore>()
            .and_then(|store| store.records.get_mut(&realm_id))
            .and_then(|record| {
                if record.closed {
                    return None;
                }
                if repeating {
                    let timer = record.intervals.get_mut(&id)?;
                    let snapshot = timer.clone();
                    timer.due_ms += timer.delay_ms.max(1.0);
                    Some(snapshot)
                } else {
                    record.timeouts.remove(&id)
                }
            });
        let Some(timer) = timer else {
            continue;
        };
        if let Some(record) = scope
            .get_slot_mut::<WorkerRealmStore>()
            .and_then(|store| store.records.get_mut(&realm_id))
        {
            record.running_timer_nesting_level = timer.nesting_level;
        }
        run_timer(scope, realm_id, timer);
        if let Some(record) = scope
            .get_slot_mut::<WorkerRealmStore>()
            .and_then(|store| store.records.get_mut(&realm_id))
        {
            record.running_timer_nesting_level = 0;
        }
        ran = true;
    }

    let mut animation_frames = scope
        .get_slot::<WorkerRealmStore>()
        .map(|store| {
            let mut callbacks = Vec::new();
            for (realm_id, record) in &store.records {
                if !record.closed
                    && record
                        .animation_frame_due_ms
                        .is_some_and(|due_ms| due_ms <= now)
                {
                    callbacks.extend(record.animation_frames.keys().map(|id| (*realm_id, *id)));
                }
            }
            callbacks
        })
        .unwrap_or_default();
    animation_frames.sort_unstable();
    for (realm_id, id) in animation_frames {
        let callback = scope
            .get_slot_mut::<WorkerRealmStore>()
            .and_then(|store| store.records.get_mut(&realm_id))
            .and_then(|record| {
                if record.closed {
                    return None;
                }
                let callback = record.animation_frames.remove(&id);
                if record.animation_frames.is_empty() {
                    record.animation_frame_due_ms = None;
                }
                callback
            });
        let Some(callback) = callback else {
            continue;
        };
        run_animation_frame(scope, realm_id, callback, now);
        ran = true;
    }
    ran
}

fn run_timer(scope: &mut v8::PinScope<'_, '_>, realm_id: i32, timer: TimerRecord) {
    let Some(record) = record(scope, realm_id).filter(|record| !record.closed) else {
        return;
    };
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    match timer.callback {
        TimerCallback::Function(callback) => {
            let callback = v8::Local::new(worker_scope, &callback);
            let receiver: v8::Local<v8::Value> = worker_scope
                .get_current_context()
                .global(worker_scope)
                .into();
            let arguments = timer
                .arguments
                .iter()
                .map(|value| v8::Local::new(worker_scope, value))
                .collect::<Vec<_>>();
            let _ = callback.call(worker_scope, receiver, &arguments);
        }
        TimerCallback::Source(source) => {
            let Some(source) = v8::String::new(worker_scope, &source) else {
                return;
            };
            if let Some(script) = v8::Script::compile(worker_scope, source, None) {
                let _ = script.run(worker_scope);
            }
        }
    }
    worker_scope.perform_microtask_checkpoint();
}

fn run_animation_frame(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    callback: v8::Global<v8::Function>,
    monotonic_time_ms: f64,
) {
    let Some(record) = record(scope, realm_id).filter(|record| !record.closed) else {
        return;
    };
    let context = v8::Local::new(scope, &record.context);
    let worker_scope = &mut v8::ContextScope::new(scope, context);
    let callback = v8::Local::new(worker_scope, &callback);
    let receiver: v8::Local<v8::Value> = worker_scope
        .get_current_context()
        .global(worker_scope)
        .into();
    let timestamp = super::performance::now_for_realm_at(worker_scope, realm_id, monotonic_time_ms)
        .unwrap_or_else(|| {
            crate::determinism::relative_high_resolution_milliseconds(
                worker_scope,
                monotonic_time_ms,
                0.0,
            )
        });
    let timestamp = v8::Number::new(worker_scope, timestamp);
    let _ = callback.call(worker_scope, receiver, &[timestamp.into()]);
    worker_scope.perform_microtask_checkpoint();
}

pub(crate) fn set_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    reserve_timer(scope, arguments, &mut result, false);
}

pub(crate) fn set_interval(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    reserve_timer(scope, arguments, &mut result, true);
}

fn reserve_timer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: &mut v8::ReturnValue<'_>,
    repeating: bool,
) {
    let callback = if let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) {
        TimerCallback::Function(v8::Global::new(scope, callback))
    } else {
        TimerCallback::Source(crate::webidl::value_to_string(scope, arguments.get(0)))
    };
    let Some(realm_id) = current_realm_id(scope) else {
        return;
    };
    let delay_ms = arguments.get(1).number_value(scope).unwrap_or(0.0);
    let nesting_level = scope
        .get_slot::<WorkerRealmStore>()
        .and_then(|store| store.records.get(&realm_id))
        .map_or(1, |record| {
            record.running_timer_nesting_level.saturating_add(1)
        });
    let delay_ms = super::timer_state::timer_delay_for_nesting(delay_ms, nesting_level);
    let values = (2..arguments.length())
        .map(|index| v8::Global::new(scope, arguments.get(index)))
        .collect();
    let due_ms = crate::determinism::elapsed_milliseconds(scope) + delay_ms;
    let Some(store) = scope.get_slot_mut::<WorkerRealmStore>() else {
        return;
    };
    let Some(record) = store.records.get_mut(&realm_id) else {
        return;
    };
    let timer = TimerRecord {
        callback,
        arguments: values,
        delay_ms,
        due_ms,
        nesting_level,
    };
    let id = record.next_timer_id;
    record.next_timer_id = record.next_timer_id.saturating_add(1).max(1);
    if repeating {
        record.intervals.insert(id, timer);
    } else {
        record.timeouts.insert(id, timer);
    }
    result.set(v8::Integer::new(scope, id).into());
}

pub(crate) fn clear_timeout(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    clear_timer(scope, arguments.get(0));
}

pub(crate) fn clear_interval(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    clear_timer(scope, arguments.get(0));
}

fn clear_timer(scope: &mut v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) {
    let id = value.int32_value(scope).unwrap_or(0);
    let Some(realm_id) = current_realm_id(scope) else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    {
        record.timeouts.remove(&id);
        record.intervals.remove(&id);
    }
}

pub(crate) fn queue_microtask(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "queueMicrotask callback must be a function");
        return;
    };
    scope.enqueue_microtask(callback);
}

pub(crate) fn request_animation_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "requestAnimationFrame callback must be a function");
        return;
    };
    let Some(realm_id) = current_realm_id(scope) else {
        return;
    };
    let saved = v8::Global::new(scope, callback);
    let due_ms = crate::determinism::elapsed_milliseconds(scope) + 1_000.0 / 60.0;
    let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    else {
        return;
    };
    let id = record.next_timer_id;
    record.next_timer_id = record.next_timer_id.saturating_add(1).max(1);
    record.animation_frames.insert(id, saved);
    record.animation_frame_due_ms.get_or_insert(due_ms);
    result.set(v8::Integer::new(scope, id).into());
}

pub(crate) fn cancel_animation_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.get(0).int32_value(scope).unwrap_or(0);
    let Some(realm_id) = current_realm_id(scope) else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<WorkerRealmStore>()
        .and_then(|store| store.records.get_mut(&realm_id))
    {
        record.animation_frames.remove(&id);
        if record.animation_frames.is_empty() {
            record.animation_frame_due_ms = None;
        }
    }
}
