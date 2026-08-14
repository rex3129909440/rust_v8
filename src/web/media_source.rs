use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaSourceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MediaSourceRecord>,
}

#[derive(Clone)]
struct MediaSourceRecord {
    source_buffers: v8::Global<v8::Object>,
    active_source_buffers: v8::Global<v8::Object>,
    duration: f64,
    ready_state: String,
    onsourceopen: Option<v8::Global<v8::Value>>,
    onsourceended: Option<v8::Global<v8::Value>>,
    onsourceclose: Option<v8::Global<v8::Value>>,
    live_seekable_range: Option<(f64, f64)>,
    handle: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaSourceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaSource", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<MediaSourceStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "MediaSource",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sourceBuffers", get_source_buffers)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "activeSourceBuffers",
        get_active_source_buffers,
    )?;
    crate::webidl::define_accessor(scope, prototype, "duration", get_duration, set_duration)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onsourceopen",
        get_onsourceopen,
        set_onsourceopen,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onsourceended",
        get_onsourceended,
        set_onsourceended,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onsourceclose",
        get_onsourceclose,
        set_onsourceclose,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readyState", get_ready_state)?;
    crate::webidl::define_method(scope, prototype, "addSourceBuffer", 1, add_source_buffer)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "clearLiveSeekableRange",
        0,
        clear_live_seekable_range,
    )?;
    crate::webidl::define_method(scope, prototype, "endOfStream", 0, end_of_stream)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "removeSourceBuffer",
        1,
        remove_source_buffer,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setLiveSeekableRange",
        2,
        set_live_seekable_range,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(
        scope,
        constructor.into(),
        "canConstructInDedicatedWorker",
        get_can_construct_in_worker,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "isTypeSupported",
        1,
        is_type_supported,
    )?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaSourceStore>()
        .ok_or_else(|| "MediaSource state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'MediaSource': Please use the 'new' operator",
        );
        return;
    }
    let source_buffers = match super::source_buffer_list::create(scope) {
        Ok(list) => list,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let active_source_buffers = match super::source_buffer_list::create(scope) {
        Ok(list) => list,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    super::event_target::attach(scope, arguments.this());
    let source_buffers = v8::Global::new(scope, source_buffers);
    let active_source_buffers = v8::Global::new(scope, active_source_buffers);
    scope
        .get_slot_mut::<MediaSourceStore>()
        .expect("MediaSource state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            MediaSourceRecord {
                source_buffers,
                active_source_buffers,
                duration: f64::NAN,
                ready_state: "closed".to_owned(),
                onsourceopen: None,
                onsourceended: None,
                onsourceclose: None,
                live_seekable_range: None,
                handle: None,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn handle(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<v8::Global<v8::Object>, String> {
    let id = object.get_identity_hash().get();
    let existing = scope
        .get_slot::<MediaSourceStore>()
        .and_then(|store| store.records.get(&id))
        .and_then(|record| record.handle.clone());
    if let Some(existing) = existing {
        return Ok(existing);
    }
    if scope
        .get_slot::<MediaSourceStore>()
        .and_then(|store| store.records.get(&id))
        .is_none()
    {
        return Err("Illegal invocation".to_owned());
    }
    let handle = super::media_source_handle::create(scope)?;
    let saved = v8::Global::new(scope, handle);
    scope
        .get_slot_mut::<MediaSourceStore>()
        .ok_or_else(|| "MediaSource state was not prepared".to_owned())?
        .records
        .get_mut(&id)
        .ok_or_else(|| "Illegal invocation".to_owned())?
        .handle = Some(saved);
    Ok(v8::Global::new(scope, handle))
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MediaSourceRecord> {
    scope
        .get_slot::<MediaSourceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MediaSourceRecord) -> &v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_source_buffers(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_list(s, a, r, |x| &x.source_buffers);
}
fn get_active_source_buffers(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_list(s, a, r, |x| &x.active_source_buffers);
}

fn get_duration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.duration).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_duration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let Some(record) = scope.get_slot_mut::<MediaSourceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.ready_state != "open" {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Failed to set the 'duration' property on 'MediaSource': The MediaSource's readyState is not 'open'.",
        );
        return;
    }
    if value.is_nan() || value < 0.0 {
        crate::webidl::throw_type_error(scope, "The duration provided is invalid");
        return;
    }
    record.duration = value;
}

fn normalized_handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    value.is_function().then(|| v8::Global::new(scope, value))
}

fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MediaSourceRecord) -> Option<&v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    update: impl FnOnce(&mut MediaSourceRecord, Option<v8::Global<v8::Value>>),
) {
    let handler = normalized_handler(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<MediaSourceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        update(record, handler);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_onsourceopen(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onsourceopen.as_ref());
}
fn set_onsourceopen(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onsourceopen = v);
}
fn get_onsourceended(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onsourceended.as_ref());
}
fn set_onsourceended(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onsourceended = v);
}
fn get_onsourceclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| x.onsourceclose.as_ref());
}
fn set_onsourceclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |x, v| x.onsourceclose = v);
}

fn get_ready_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.ready_state) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn add_source_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    if !crate::fingerprint_environment::media_capability_matches(
        &crate::fingerprint::edge(scope).media.media_source_types,
        &media_type,
    ) {
        throw_dom_exception(
            scope,
            "NotSupportedError",
            &format!(
                "Failed to execute 'addSourceBuffer' on 'MediaSource': The type provided ('{media_type}') is unsupported."
            ),
        );
        return;
    }
    if record.ready_state != "open" {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Failed to execute 'addSourceBuffer' on 'MediaSource': The MediaSource's readyState is not 'open'.",
        );
        return;
    }
    match super::source_buffer::create(scope, media_type) {
        Ok(buffer) => result.set(buffer.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn clear_live_seekable_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_open(scope, arguments.this(), |record| {
        record.live_seekable_range = None
    });
}

fn end_of_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_open(scope, arguments.this(), |record| {
        record.ready_state = "ended".to_owned()
    });
}

fn remove_source_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    throw_dom_exception(
        scope,
        "NotFoundError",
        "The SourceBuffer provided is not contained in this MediaSource.",
    );
}

fn set_live_seekable_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let start = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let end = arguments.get(1).number_value(scope).unwrap_or(f64::NAN);
    update_open(scope, arguments.this(), |record| {
        record.live_seekable_range = Some((start, end))
    });
}

fn update_open(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    update: impl FnOnce(&mut MediaSourceRecord),
) {
    let Some(record) = scope
        .get_slot_mut::<MediaSourceStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.ready_state != "open" {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "The MediaSource's readyState is not 'open'.",
        );
        return;
    }
    update(record);
}

fn get_can_construct_in_worker(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Boolean::new(scope, true).into());
}

fn is_type_supported(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let configured = &crate::fingerprint::edge(scope).media.media_source_types;
    result.set(
        v8::Boolean::new(
            scope,
            crate::fingerprint_environment::media_capability_matches(configured, &media_type),
        )
        .into(),
    );
}

fn throw_dom_exception(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), name.to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(error) => crate::webidl::throw_type_error(scope, &error),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<MediaSourceStore>() {
        store.constructor.remove(realm_id);
    }
}
