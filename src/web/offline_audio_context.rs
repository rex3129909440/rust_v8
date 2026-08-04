use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct OfflineAudioContextStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, OfflineAudioContextRecord>,
}

#[derive(Clone)]
struct OfflineAudioContextRecord {
    number_of_channels: u32,
    length: u32,
    sample_rate: f64,
    oncomplete: Option<v8::Global<v8::Value>>,
    rendered_buffer: Option<v8::Global<v8::Object>>,
    rendering_resolver: Option<v8::Global<v8::PromiseResolver>>,
    suspensions: Vec<OfflineSuspension>,
    rendering_started: bool,
    rendering_finished: bool,
}

#[derive(Clone)]
struct OfflineSuspension {
    frame: u64,
    resolver: v8::Global<v8::PromiseResolver>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OfflineAudioContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "OfflineAudioContext", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<OfflineAudioContextStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "OfflineAudioContext",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncomplete",
        get_oncomplete,
        set_oncomplete,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "resume", 0, resume)?;
    crate::webidl::define_method(scope, prototype, "startRendering", 0, start_rendering)?;
    crate::webidl::define_method(scope, prototype, "suspend", 1, suspend)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::base_audio_context::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<OfflineAudioContextStore>()
        .ok_or_else(|| "OfflineAudioContext state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'OfflineAudioContext': 1 argument required",
        );
        return;
    }
    let configuration = if let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        let channels =
            super::event::number_property(scope, options, "numberOfChannels", 1.0) as u32;
        let length = super::event::number_property(scope, options, "length", 0.0) as u32;
        let sample_rate = super::event::number_property(scope, options, "sampleRate", 0.0);
        Some((channels, length, sample_rate))
    } else if arguments.length() >= 3 {
        Some((
            arguments.get(0).uint32_value(scope).unwrap_or(0),
            arguments.get(1).uint32_value(scope).unwrap_or(0),
            arguments.get(2).number_value(scope).unwrap_or(0.0),
        ))
    } else {
        None
    };
    let Some((channels, length, sample_rate)) = configuration else {
        crate::webidl::throw_type_error(
            scope,
            "The OfflineAudioContext configuration is incomplete",
        );
        return;
    };
    if !(1..=32).contains(&channels)
        || length == 0
        || !sample_rate.is_finite()
        || !(3_000.0..=768_000.0).contains(&sample_rate)
    {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The OfflineAudioContext configuration is outside the supported range".to_owned(),
            "NotSupportedError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    match attach(scope, arguments.this(), channels, length, sample_rate) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
    number_of_channels: u32,
    length: u32,
    sample_rate: f64,
) -> Result<(), String> {
    super::base_audio_context::attach(scope, context, sample_rate, "suspended", true)?;
    scope
        .get_slot_mut::<OfflineAudioContextStore>()
        .ok_or_else(|| "OfflineAudioContext state was not prepared".to_owned())?
        .records
        .insert(
            context.get_identity_hash().get(),
            OfflineAudioContextRecord {
                number_of_channels,
                length,
                sample_rate,
                oncomplete: None,
                rendered_buffer: None,
                rendering_resolver: None,
                suspensions: Vec::new(),
                rendering_started: false,
                rendering_finished: false,
            },
        );
    Ok(())
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<OfflineAudioContextRecord> {
    scope
        .get_slot::<OfflineAudioContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_offline_context(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<OfflineAudioContextStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

fn get_oncomplete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.oncomplete {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn set_oncomplete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    let present = value.is_some();
    if let Some(record) = scope
        .get_slot_mut::<OfflineAudioContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.oncomplete = value;
        super::event_target::set_attribute_handler(scope, arguments.this(), "complete", present);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn dispatch_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "complete" {
        return;
    }
    let handler = scope
        .get_slot::<OfflineAudioContextStore>()
        .and_then(|store| store.records.get(&target.get_identity_hash().get()))
        .and_then(|record| record.oncomplete.clone());
    let Some(handler) = handler else {
        return;
    };
    let Ok(handler) = v8::Local::<v8::Function>::try_from(v8::Local::new(scope, &handler)) else {
        return;
    };
    let _ = handler.call(scope, target.into(), &[event.into()]);
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn resolved_undefined(scope: &mut v8::PinScope<'_, '_>, mut result: v8::ReturnValue<'_>) {
    if let Ok(promise) =
        super::writable_stream::resolved_promise(scope, v8::undefined(scope).into())
    {
        result.set(promise.into());
    }
}

fn resume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !snapshot.rendering_started || snapshot.rendering_finished {
        rejected_invalid_state(
            scope,
            result,
            "Failed to execute 'resume' on 'OfflineAudioContext': cannot resume an offline context that has not started",
        );
        return;
    }
    if super::base_audio_context::state(scope, arguments.this()).as_deref() == Some("suspended") {
        super::base_audio_context::set_state(scope, arguments.this(), "running");
        queue_rendering_step(scope, arguments.this());
    }
    resolved_undefined(scope, result);
}

fn suspend(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.rendering_finished {
        rejected_invalid_state(
            scope,
            result,
            "Failed to execute 'suspend' on 'OfflineAudioContext': the rendering is already finished",
        );
        return;
    }
    let when = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !when.is_finite() || when < 0.0 || when > record.length as f64 / record.sample_rate {
        crate::webidl::throw_type_error(scope, "The suspend time is outside the render duration");
        return;
    }
    let frame =
        ((when * record.sample_rate / 128.0).ceil() as u64 * 128).min(u64::from(record.length));
    let current_frame = (super::base_audio_context::current_time(scope, arguments.this())
        .unwrap_or(0.0)
        * record.sample_rate)
        .round() as u64;
    if record.rendering_started && frame <= current_frame {
        rejected_invalid_state(
            scope,
            result,
            "Failed to execute 'suspend' on 'OfflineAudioContext': the suspend time is earlier than the current render position",
        );
        return;
    }
    if record
        .suspensions
        .iter()
        .any(|suspension| suspension.frame == frame)
    {
        rejected_invalid_state(
            scope,
            result,
            "Failed to execute 'suspend' on 'OfflineAudioContext': a suspension is already scheduled at this time",
        );
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let promise = resolver.get_promise(scope);
    let resolver = v8::Global::new(scope, resolver);
    if let Some(record) = scope
        .get_slot_mut::<OfflineAudioContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record
            .suspensions
            .push(OfflineSuspension { frame, resolver });
        record
            .suspensions
            .sort_by_key(|suspension| suspension.frame);
        result.set(promise.into());
    }
}

fn start_rendering(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.rendering_started {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "Rendering has already started".to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let buffer = match super::audio_buffer::create(
        scope,
        snapshot.number_of_channels,
        snapshot.length,
        snapshot.sample_rate,
    ) {
        Ok(buffer) => buffer,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    super::audio_render::render(
        scope,
        arguments.this(),
        buffer,
        snapshot.number_of_channels,
        snapshot.length,
        snapshot.sample_rate,
    );
    super::audio_buffer::apply_fingerprint_noise(scope, buffer);
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    let promise = resolver.get_promise(scope);
    let buffer_global = v8::Global::new(scope, buffer);
    let resolver_global = v8::Global::new(scope, resolver);
    if let Some(record) = scope
        .get_slot_mut::<OfflineAudioContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.rendering_started = true;
        record.rendered_buffer = Some(buffer_global);
        record.rendering_resolver = Some(resolver_global);
    }
    super::base_audio_context::set_state(scope, arguments.this(), "running");
    queue_rendering_step(scope, arguments.this());
    result.set(promise.into());
}

fn queue_rendering_step(scope: &mut v8::PinScope<'_, '_>, context: v8::Local<'_, v8::Object>) {
    let task = v8::Function::builder(advance_rendering)
        .data(context.into())
        .constructor_behavior(v8::ConstructorBehavior::Throw)
        .build(scope);
    if let Some(task) = task {
        scope.enqueue_microtask(task);
    }
}

fn advance_rendering(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.data()) else {
        return;
    };
    let Some(snapshot) = record(scope, context) else {
        return;
    };
    if snapshot.rendering_finished
        || super::base_audio_context::state(scope, context).as_deref() != Some("running")
    {
        return;
    }
    let current_frame = (super::base_audio_context::current_time(scope, context).unwrap_or(0.0)
        * snapshot.sample_rate)
        .round() as u64;
    let suspension = snapshot
        .suspensions
        .iter()
        .find(|suspension| suspension.frame >= current_frame)
        .cloned();
    if let Some(suspension) = suspension
        && suspension.frame < u64::from(snapshot.length)
    {
        super::base_audio_context::advance_offline_to_frame(scope, context, suspension.frame);
        if let Some(record) = scope
            .get_slot_mut::<OfflineAudioContextStore>()
            .and_then(|store| store.records.get_mut(&context.get_identity_hash().get()))
        {
            record
                .suspensions
                .retain(|entry| entry.frame != suspension.frame);
        }
        super::base_audio_context::set_state(scope, context, "suspended");
        let resolver = v8::Local::new(scope, &suspension.resolver);
        let undefined = v8::undefined(scope);
        let _ = resolver.resolve(scope, undefined.into());
        return;
    }

    super::base_audio_context::advance_offline_to_frame(scope, context, u64::from(snapshot.length));
    super::base_audio_context::set_state(scope, context, "closed");
    if let Some(record) = scope
        .get_slot_mut::<OfflineAudioContextStore>()
        .and_then(|store| store.records.get_mut(&context.get_identity_hash().get()))
    {
        record.rendering_finished = true;
    }
    let Some(buffer) = snapshot.rendered_buffer else {
        return;
    };
    let buffer = v8::Local::new(scope, &buffer);
    if let Some(resolver) = snapshot.rendering_resolver {
        let resolver = v8::Local::new(scope, &resolver);
        let _ = resolver.resolve(scope, buffer.into());
    }
    let Ok(event) = super::offline_audio_completion_event::create(scope, "complete", buffer) else {
        return;
    };
    super::event_target::dispatch(scope, context, event);
}

fn rejected_invalid_state(
    scope: &mut v8::PinScope<'_, '_>,
    mut result: v8::ReturnValue<'_>,
    message: &str,
) {
    let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "InvalidStateError".to_owned())
    else {
        return;
    };
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception.into()) {
        result.set(promise.into());
    }
}
