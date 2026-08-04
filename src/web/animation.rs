use std::collections::HashMap;

#[derive(Clone)]
struct AnimationRecord {
    object: v8::Global<v8::Object>,
    sequence: u64,
    effect: Option<v8::Global<v8::Value>>,
    timeline: Option<v8::Global<v8::Value>>,
    start_time: Option<f64>,
    current_time: Option<f64>,
    playback_rate: f64,
    range_start: String,
    range_end: String,
    play_state: String,
    replace_state: String,
    pending: bool,
    id: String,
    onfinish: Option<v8::Global<v8::Value>>,
    oncancel: Option<v8::Global<v8::Value>>,
    onremove: Option<v8::Global<v8::Value>>,
    finished: v8::Global<v8::Promise>,
    ready: v8::Global<v8::Promise>,
}

#[derive(Default)]
pub(crate) struct AnimationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AnimationRecord>,
    next_sequence: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationStore::default());
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<AnimationStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Animation", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AnimationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Animation",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "effect", get_effect, set_effect)?;
    crate::webidl::define_accessor(scope, prototype, "timeline", get_timeline, set_timeline)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "startTime",
        get_start_time,
        set_start_time,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "currentTime",
        get_current_time,
        set_current_time,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "playbackRate",
        get_playback_rate,
        set_playback_rate,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "rangeStart",
        get_range_start,
        set_range_start,
    )?;
    crate::webidl::define_accessor(scope, prototype, "rangeEnd", get_range_end, set_range_end)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "playState", get_play_state)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "replaceState", get_replace_state)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pending", get_pending)?;
    crate::webidl::define_accessor(scope, prototype, "id", get_id, set_id)?;
    crate::webidl::define_accessor(scope, prototype, "onfinish", get_onfinish, set_onfinish)?;
    crate::webidl::define_accessor(scope, prototype, "oncancel", get_oncancel, set_oncancel)?;
    crate::webidl::define_accessor(scope, prototype, "onremove", get_onremove, set_onremove)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "finished", get_finished)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ready", get_ready)?;
    crate::webidl::define_method(scope, prototype, "cancel", 0, cancel)?;
    crate::webidl::define_method(scope, prototype, "commitStyles", 0, commit_styles)?;
    crate::webidl::define_method(scope, prototype, "finish", 0, finish)?;
    crate::webidl::define_method(scope, prototype, "pause", 0, pause)?;
    crate::webidl::define_method(scope, prototype, "persist", 0, persist)?;
    crate::webidl::define_method(scope, prototype, "play", 0, play)?;
    crate::webidl::define_method(scope, prototype, "reverse", 0, reverse)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "updatePlaybackRate",
        1,
        update_playback_rate,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "overallProgress",
        get_overall_progress,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AnimationStore>()
        .ok_or_else(|| "Animation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn resolved_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let resolver = v8::PromiseResolver::new(scope)
        .ok_or_else(|| "cannot create Animation promise".to_owned())?;
    let _ = resolver.resolve(scope, value);
    Ok(resolver.get_promise(scope))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Animation must be constructed");
        return;
    }
    let effect = (!arguments.get(0).is_null_or_undefined())
        .then(|| v8::Global::new(scope, arguments.get(0)));
    let timeline_value = if arguments.length() < 2 || arguments.get(1).is_undefined() {
        default_timeline(scope).unwrap_or_else(|| v8::null(scope).into())
    } else {
        arguments.get(1)
    };
    let timeline =
        (!timeline_value.is_null_or_undefined()).then(|| v8::Global::new(scope, timeline_value));
    let promise_value: v8::Local<v8::Value> = arguments.this().into();
    let Ok(finished) = resolved_promise(scope, promise_value) else {
        return;
    };
    let Ok(ready) = resolved_promise(scope, promise_value) else {
        return;
    };
    super::event_target::attach(scope, arguments.this());
    let sequence = scope
        .get_slot::<AnimationStore>()
        .map(|store| store.next_sequence)
        .unwrap_or_default();
    let record = AnimationRecord {
        object: v8::Global::new(scope, arguments.this()),
        sequence,
        effect,
        timeline,
        start_time: None,
        current_time: None,
        playback_rate: 1.0,
        range_start: "normal".to_owned(),
        range_end: "normal".to_owned(),
        play_state: "idle".to_owned(),
        replace_state: "active".to_owned(),
        pending: false,
        id: String::new(),
        onfinish: None,
        oncancel: None,
        onremove: None,
        finished: v8::Global::new(scope, finished),
        ready: v8::Global::new(scope, ready),
    };
    let store = scope
        .get_slot_mut::<AnimationStore>()
        .expect("Animation state");
    store.next_sequence = sequence.saturating_add(1);
    store
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn default_timeline<'s>(scope: &v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::Value>> {
    let global = scope.get_current_context().global(scope);
    let document_key = v8::String::new(scope, "document")?;
    let document = global.get(scope, document_key.into())?;
    let document = v8::Local::<v8::Object>::try_from(document).ok()?;
    let timeline_key = v8::String::new(scope, "timeline")?;
    document.get(scope, timeline_key.into())
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AnimationRecord> {
    scope
        .get_slot::<AnimationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn active_targets(
    scope: &v8::PinScope<'_, '_>,
) -> Vec<(u64, v8::Global<v8::Object>, v8::Global<v8::Object>)> {
    let mut values = scope
        .get_slot::<AnimationStore>()
        .map(|store| {
            store
                .records
                .values()
                .filter(|record| record.play_state != "idle" && record.replace_state != "removed")
                .filter_map(|record| {
                    let effect = record.effect.as_ref()?;
                    let effect = v8::Local::new(scope, effect);
                    let effect = v8::Local::<v8::Object>::try_from(effect).ok()?;
                    let target = super::keyframe_effect::target(scope, effect)?;
                    Some((record.sequence, record.object.clone(), target))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    values.sort_by_key(|(sequence, _, _)| *sequence);
    values
}

pub(crate) fn for_element(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    subtree: bool,
) -> Vec<v8::Global<v8::Object>> {
    active_targets(scope)
        .into_iter()
        .filter(|(_, _, target)| {
            let target = v8::Local::new(scope, target);
            if subtree {
                super::node::is_descendant(scope, element, target)
            } else {
                target.get_identity_hash().get() == element.get_identity_hash().get()
            }
        })
        .map(|(_, animation, _)| animation)
        .collect()
}

pub(crate) fn for_document(
    scope: &v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Vec<v8::Global<v8::Object>> {
    active_targets(scope)
        .into_iter()
        .filter(|(_, _, target)| {
            let target = v8::Local::new(scope, target);
            super::node::is_connected(scope, target)
                && super::node::owner_document(scope, target).is_some_and(|owner| {
                    owner.get_identity_hash().get() == document.get_identity_hash().get()
                })
        })
        .map(|(_, animation, _)| animation)
        .collect()
}

fn set_record(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    update: impl FnOnce(&mut AnimationRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<AnimationStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        update(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_nullable_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<Option<v8::Global<v8::Value>>>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Some(Some(value)) => result.set(v8::Local::new(scope, &value)),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn get_effect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_nullable_value(s, record(s, a.this()).map(|v| v.effect), r);
}
fn set_effect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = (!a.get(0).is_null_or_undefined()).then(|| v8::Global::new(s, a.get(0)));
    set_record(s, a.this(), |v| v.effect = value);
}
fn get_timeline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_nullable_value(s, record(s, a.this()).map(|v| v.timeline), r);
}
fn set_timeline(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = (!a.get(0).is_null_or_undefined()).then(|| v8::Global::new(s, a.get(0)));
    set_record(s, a.this(), |v| v.timeline = value);
}

fn get_optional_number(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<Option<f64>>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Some(Some(value)) => result.set(v8::Number::new(scope, value).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn number_or_none(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<f64> {
    if value.is_null() {
        None
    } else {
        value.number_value(scope).filter(|value| value.is_finite())
    }
}

fn get_start_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_optional_number(s, record(s, a.this()).map(|v| v.start_time), r);
}
fn set_start_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_or_none(s, a.get(0));
    set_record(s, a.this(), |v| v.start_time = value);
}
fn get_current_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_optional_number(s, record(s, a.this()).map(|v| v.current_time), r);
}
fn set_current_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = number_or_none(s, a.get(0));
    set_record(s, a.this(), |v| v.current_time = value);
}

fn get_playback_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.playback_rate).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_playback_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if value.is_finite() {
        set_record(scope, arguments.this(), |record| {
            record.playback_rate = value
        });
    }
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<String>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value
        && let Some(value) = v8::String::new(scope, &value)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_range_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.range_start);
    return_string(s, value, r);
}
fn set_range_start(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    set_record(s, a.this(), |v| v.range_start = value);
}
fn get_range_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.range_end);
    return_string(s, value, r);
}
fn set_range_end(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    set_record(s, a.this(), |v| v.range_end = value);
}
fn get_play_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.play_state);
    return_string(s, value, r);
}
fn get_replace_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.replace_state);
    return_string(s, value, r);
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.id);
    return_string(s, value, r);
}
fn set_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    set_record(s, a.this(), |v| v.id = value);
}

fn get_pending(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.pending).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<Option<v8::Global<v8::Value>>>,
    mut result: v8::ReturnValue<'_>,
) {
    match value {
        Some(Some(value)) => result.set(v8::Local::new(scope, &value)),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

fn handler_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    value.is_function().then(|| v8::Global::new(scope, value))
}

fn get_onfinish(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, record(s, a.this()).map(|v| v.onfinish), r);
}
fn set_onfinish(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(s, a.get(0));
    set_record(s, a.this(), |v| v.onfinish = value);
}
fn get_oncancel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, record(s, a.this()).map(|v| v.oncancel), r);
}
fn set_oncancel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(s, a.get(0));
    set_record(s, a.this(), |v| v.oncancel = value);
}
fn get_onremove(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, record(s, a.this()).map(|v| v.onremove), r);
}
fn set_onremove(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler_value(s, a.get(0));
    set_record(s, a.this(), |v| v.onremove = value);
}

fn get_finished(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.finished).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_ready(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.ready).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn cancel(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_record(scope, a.this(), |v| {
        v.play_state = "idle".to_owned();
        v.current_time = None;
        v.start_time = None;
        v.pending = false;
    });
}
fn commit_styles(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    set_record(scope, arguments.this(), |animation| {
        animation.pending = false;
    });
    result.set(v8::undefined(scope).into());
}
fn finish(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_record(scope, a.this(), |v| {
        v.play_state = "finished".to_owned();
        v.pending = false;
    });
}
fn pause(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_record(scope, a.this(), |v| {
        v.play_state = "paused".to_owned();
        v.pending = false;
    });
}
fn persist(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_record(scope, a.this(), |v| {
        v.replace_state = "persisted".to_owned()
    });
}
fn play(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_record(scope, a.this(), |v| {
        v.play_state = "running".to_owned();
        v.pending = false;
    });
}
fn reverse(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_record(scope, a.this(), |v| {
        v.playback_rate = -v.playback_rate;
        v.play_state = "running".to_owned();
    });
}
fn update_playback_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if value.is_finite() {
        set_record(scope, arguments.this(), |record| {
            record.playback_rate = value
        });
    }
}

fn get_overall_progress(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.current_time {
            Some(value) => result.set(v8::Number::new(scope, value.max(0.0)).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
