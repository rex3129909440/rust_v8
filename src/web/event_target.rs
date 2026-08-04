use std::collections::{HashMap, HashSet};

pub(crate) struct EventTargetStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) targets: HashMap<i32, EventTargetRecord>,
    aliases: HashMap<i32, i32>,
    pub(crate) next_listener_id: u64,
}

impl Default for EventTargetStore {
    fn default() -> Self {
        Self {
            constructors: HashMap::new(),
            targets: HashMap::new(),
            aliases: HashMap::new(),
            next_listener_id: 1,
        }
    }
}

#[derive(Default)]
pub(crate) struct EventTargetRecord {
    pub(crate) listeners: HashMap<String, Vec<EventListener>>,
    pub(crate) attribute_handlers: HashMap<String, u64>,
    pub(crate) waiters: HashMap<String, Vec<v8::Global<v8::PromiseResolver>>>,
}

#[derive(Clone)]
pub(crate) struct EventListener {
    pub(crate) registration_id: u64,
    pub(crate) identity: i32,
    pub(crate) callback: v8::Global<v8::Object>,
    pub(crate) capture: bool,
    pub(crate) once: bool,
    pub(crate) passive: bool,
    pub(crate) signal_identity: Option<i32>,
}

#[derive(Clone)]
pub(crate) enum Invocation {
    Listener(EventListener),
    Attribute(u64),
}

#[derive(Default)]
pub(crate) struct ListenerOptions {
    pub(crate) capture: bool,
    pub(crate) once: bool,
    pub(crate) passive: bool,
    pub(crate) signal: Option<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(EventTargetStore::default());
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<EventTargetStore>()
        .and_then(|store| store.constructors.get(&realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "EventTarget",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::event_target_add_event_listener::define(scope, prototype)?;
    super::event_target_dispatch_event::define(scope, prototype)?;
    super::event_target_remove_event_listener::define(scope, prototype)?;
    super::event_target_when::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<EventTargetStore>()
        .ok_or_else(|| "EventTarget state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "EventTarget", constructor.into())
}

pub(crate) fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    if let Some(store) = scope.get_slot_mut::<EventTargetStore>() {
        store
            .targets
            .entry(object.get_identity_hash().get())
            .or_default();
    }
}

pub(crate) fn attach_alias(
    scope: &mut v8::PinScope<'_, '_>,
    alias: v8::Local<'_, v8::Object>,
    target: v8::Local<'_, v8::Object>,
) {
    let alias_id = alias.get_identity_hash().get();
    let target_id = target_record_id(scope, target);
    if let Some(store) = scope.get_slot_mut::<EventTargetStore>() {
        store.aliases.insert(alias_id, target_id);
    }
}

pub(crate) fn target_record_id(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> i32 {
    let identity = object.get_identity_hash().get();
    scope
        .get_slot::<EventTargetStore>()
        .and_then(|store| store.aliases.get(&identity))
        .copied()
        .unwrap_or(identity)
}

pub(crate) fn reset(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let identity = target_record_id(scope, object);
    if let Some(store) = scope.get_slot_mut::<EventTargetStore>() {
        store.targets.insert(identity, EventTargetRecord::default());
    }
}

pub(crate) fn create_event<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
) -> v8::Local<'s, v8::Object> {
    if let Ok(event) = super::event::create(scope, event_type) {
        return event;
    }
    let event = v8::Object::new(scope);
    define_data(scope, event, "type", string_value(scope, event_type));
    define_data(
        scope,
        event,
        "defaultPrevented",
        v8::Boolean::new(scope, false).into(),
    );
    event
}

pub(crate) fn dispatch(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
) -> bool {
    let target_id = target_record_id(scope, target);
    if !scope
        .get_slot::<EventTargetStore>()
        .is_some_and(|store| store.targets.contains_key(&target_id))
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    }
    if !super::event::is_event(scope, event) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'dispatchEvent' on 'EventTarget': parameter 1 is not of type 'Event'.",
        );
        return false;
    }
    let event_type = super::event::event_type(scope, event).unwrap_or_default();
    if super::event::is_dispatching(scope, event) {
        throw_invalid_state(scope);
        return false;
    }
    let path = propagation_path(scope, target, super::event::composed(scope, event));
    if !super::event::begin_dispatch(scope, event, target, &path) {
        throw_invalid_state(scope);
        return false;
    }
    super::event_global::begin_dispatch(scope, event);

    for (index, ancestor) in path.iter().enumerate().skip(1).rev() {
        if super::event::propagation_stopped(scope, event) {
            break;
        }
        let ancestor = v8::Local::new(scope, ancestor);
        let exposed_target = retargeted_target(scope, &path, index);
        let phase =
            if ancestor.get_identity_hash().get() == exposed_target.get_identity_hash().get() {
                super::event::AT_TARGET
            } else {
                super::event::CAPTURING_PHASE
            };
        super::event::set_current_target(scope, event, ancestor, phase, exposed_target);
        invoke_listeners(scope, ancestor, event, &event_type, true);
    }

    if !super::event::propagation_stopped(scope, event) {
        super::event::set_current_target(scope, event, target, super::event::AT_TARGET, target);
        invoke_listeners(scope, target, event, &event_type, true);
        if !super::event::immediate_propagation_stopped(scope, event) {
            invoke_listeners(scope, target, event, &event_type, false);
        }
        resolve_waiters(scope, target, event, &event_type);
    }

    for (index, ancestor) in path.iter().enumerate().skip(1) {
        if super::event::propagation_stopped(scope, event) {
            break;
        }
        let ancestor = v8::Local::new(scope, ancestor);
        let exposed_target = retargeted_target(scope, &path, index);
        let at_adjusted_target =
            ancestor.get_identity_hash().get() == exposed_target.get_identity_hash().get();
        if !super::event::bubbles(scope, event) && !at_adjusted_target {
            continue;
        }
        let phase = if at_adjusted_target {
            super::event::AT_TARGET
        } else {
            super::event::BUBBLING_PHASE
        };
        super::event::set_current_target(scope, event, ancestor, phase, exposed_target);
        invoke_listeners(scope, ancestor, event, &event_type, false);
        resolve_waiters(scope, ancestor, event, &event_type);
    }

    let allowed = !super::event::default_prevented(scope, event).unwrap_or(false);
    let final_target = retargeted_target(scope, &path, path.len().saturating_sub(1));
    super::event::set_target(scope, event, final_target);
    super::event_global::finish_dispatch(scope);
    super::event::finish_dispatch(scope, event);
    allowed
}

pub(crate) fn invoke_attribute_handlers(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    super::abort_signal::dispatch_handler(scope, target, event, event_type);
    super::base_audio_context::dispatch_handler(scope, target, event, event_type);
    super::audio_scheduled_source_node::dispatch_handler(scope, target, event, event_type);
    super::offline_audio_context::dispatch_handler(scope, target, event, event_type);
    super::performance::dispatch_handler(scope, target, event, event_type);
    super::element::dispatch_handler(scope, target, event, event_type);
    super::html_element::dispatch_handler(scope, target, event, &event_type);
    super::svg_element::dispatch_handler(scope, target, event, &event_type);
    super::document::dispatch_handler(scope, target, event, &event_type);
    super::cookie_store::dispatch_handler(scope, target, event, &event_type);
    super::on_search::dispatch(scope, target, event, &event_type);
    super::on_app_installed::dispatch(scope, target, event, &event_type);
    super::on_before_install_prompt::dispatch(scope, target, event, &event_type);
    super::on_abort::dispatch(scope, target, event, &event_type);
    super::on_before_input::dispatch(scope, target, event, &event_type);
    super::on_before_match::dispatch(scope, target, event, &event_type);
    super::on_before_toggle::dispatch(scope, target, event, &event_type);
    super::on_blur::dispatch(scope, target, event, &event_type);
    super::on_cancel::dispatch(scope, target, event, &event_type);
    super::on_can_play::dispatch(scope, target, event, &event_type);
    super::on_can_play_through::dispatch(scope, target, event, &event_type);
    super::on_change::dispatch(scope, target, event, &event_type);
    super::on_click::dispatch(scope, target, event, &event_type);
    super::on_close::dispatch(scope, target, event, &event_type);
    super::on_command::dispatch(scope, target, event, &event_type);
    super::on_content_visibility_auto_state_change::dispatch(scope, target, event, &event_type);
    super::on_context_lost::dispatch(scope, target, event, &event_type);
    super::on_context_menu::dispatch(scope, target, event, &event_type);
    super::on_context_restored::dispatch(scope, target, event, &event_type);
    super::on_cue_change::dispatch(scope, target, event, &event_type);
    super::on_double_click::dispatch(scope, target, event, &event_type);
    super::on_drag::dispatch(scope, target, event, &event_type);
    super::on_drag_end::dispatch(scope, target, event, &event_type);
    super::on_drag_enter::dispatch(scope, target, event, &event_type);
    super::on_drag_leave::dispatch(scope, target, event, &event_type);
    super::on_drag_over::dispatch(scope, target, event, &event_type);
    super::on_drag_start::dispatch(scope, target, event, &event_type);
    super::on_drop::dispatch(scope, target, event, &event_type);
    super::on_device_motion::dispatch(scope, target, event, &event_type);
    super::on_device_orientation::dispatch(scope, target, event, &event_type);
    super::on_device_orientation_absolute::dispatch(scope, target, event, &event_type);
    super::on_duration_change::dispatch(scope, target, event, &event_type);
    super::on_emptied::dispatch(scope, target, event, &event_type);
    super::on_ended::dispatch(scope, target, event, &event_type);
    super::on_error::dispatch(scope, target, event, &event_type);
    super::on_focus::dispatch(scope, target, event, &event_type);
    super::on_form_data::dispatch(scope, target, event, &event_type);
    super::on_gamepad_connected::dispatch(scope, target, event, &event_type);
    super::on_gamepad_disconnected::dispatch(scope, target, event, &event_type);
    super::on_input::dispatch(scope, target, event, &event_type);
    super::on_invalid::dispatch(scope, target, event, &event_type);
    super::on_key_down::dispatch(scope, target, event, &event_type);
    super::on_key_press::dispatch(scope, target, event, &event_type);
    super::on_key_up::dispatch(scope, target, event, &event_type);
    super::on_load::dispatch(scope, target, event, &event_type);
    super::on_loaded_data::dispatch(scope, target, event, &event_type);
    super::on_loaded_metadata::dispatch(scope, target, event, &event_type);
    super::on_load_start::dispatch(scope, target, event, &event_type);
    super::on_mouse_down::dispatch(scope, target, event, &event_type);
    super::on_mouse_enter::dispatch(scope, target, event, &event_type);
    super::on_mouse_leave::dispatch(scope, target, event, &event_type);
    super::on_mouse_move::dispatch(scope, target, event, &event_type);
    super::on_mouse_out::dispatch(scope, target, event, &event_type);
    super::on_mouse_over::dispatch(scope, target, event, &event_type);
    super::on_mouse_up::dispatch(scope, target, event, &event_type);
    super::on_mouse_wheel::dispatch(scope, target, event, &event_type);
    super::on_pause::dispatch(scope, target, event, &event_type);
    super::on_play::dispatch(scope, target, event, &event_type);
    super::on_playing::dispatch(scope, target, event, &event_type);
    super::on_progress::dispatch(scope, target, event, &event_type);
    super::on_rate_change::dispatch(scope, target, event, &event_type);
    super::on_reset::dispatch(scope, target, event, &event_type);
    super::on_resize::dispatch(scope, target, event, &event_type);
    super::on_scroll::dispatch(scope, target, event, &event_type);
    super::on_scroll_end::dispatch(scope, target, event, &event_type);
    super::on_scroll_snap_change::dispatch(scope, target, event, &event_type);
    super::on_scroll_snap_changing::dispatch(scope, target, event, &event_type);
    super::on_security_policy_violation::dispatch(scope, target, event, &event_type);
    super::on_seeked::dispatch(scope, target, event, &event_type);
    super::on_seeking::dispatch(scope, target, event, &event_type);
    super::on_select::dispatch(scope, target, event, &event_type);
    super::on_slot_change::dispatch(scope, target, event, &event_type);
    super::on_stalled::dispatch(scope, target, event, &event_type);
    super::on_submit::dispatch(scope, target, event, &event_type);
    super::on_suspend::dispatch(scope, target, event, &event_type);
    super::on_time_update::dispatch(scope, target, event, &event_type);
    super::on_toggle::dispatch(scope, target, event, &event_type);
    super::on_volume_change::dispatch(scope, target, event, &event_type);
    super::on_waiting::dispatch(scope, target, event, &event_type);
    super::on_webkit_animation_end::dispatch(scope, target, event, &event_type);
    super::on_webkit_animation_iteration::dispatch(scope, target, event, &event_type);
    super::on_webkit_animation_start::dispatch(scope, target, event, &event_type);
    super::on_webkit_transition_end::dispatch(scope, target, event, &event_type);
    super::on_wheel::dispatch(scope, target, event, &event_type);
    super::on_aux_click::dispatch(scope, target, event, &event_type);
    super::on_got_pointer_capture::dispatch(scope, target, event, &event_type);
    super::on_lost_pointer_capture::dispatch(scope, target, event, &event_type);
    super::on_pointer_down::dispatch(scope, target, event, &event_type);
    super::on_pointer_move::dispatch(scope, target, event, &event_type);
    super::on_pointer_up::dispatch(scope, target, event, &event_type);
    super::on_pointer_cancel::dispatch(scope, target, event, &event_type);
    super::on_pointer_over::dispatch(scope, target, event, &event_type);
    super::on_pointer_out::dispatch(scope, target, event, &event_type);
    super::on_pointer_enter::dispatch(scope, target, event, &event_type);
    super::on_pointer_leave::dispatch(scope, target, event, &event_type);
    super::on_pointer_raw_update::dispatch(scope, target, event, &event_type);
    super::on_select_start::dispatch(scope, target, event, &event_type);
    super::on_selection_change::dispatch(scope, target, event, &event_type);
    super::on_animation_cancel::dispatch(scope, target, event, &event_type);
    super::on_animation_end::dispatch(scope, target, event, &event_type);
    super::on_animation_iteration::dispatch(scope, target, event, &event_type);
    super::on_animation_start::dispatch(scope, target, event, &event_type);
    super::on_transition_run::dispatch(scope, target, event, &event_type);
    super::on_transition_start::dispatch(scope, target, event, &event_type);
    super::on_transition_end::dispatch(scope, target, event, &event_type);
    super::on_transition_cancel::dispatch(scope, target, event, &event_type);
    super::on_before_xr_select::dispatch(scope, target, event, &event_type);
    super::on_after_print::dispatch(scope, target, event, &event_type);
    super::on_before_print::dispatch(scope, target, event, &event_type);
    super::on_before_unload::dispatch(scope, target, event, &event_type);
    super::on_hash_change::dispatch(scope, target, event, &event_type);
    super::on_language_change::dispatch(scope, target, event, &event_type);
    super::on_message::dispatch(scope, target, event, &event_type);
    super::on_message_error::dispatch(scope, target, event, &event_type);
    super::on_offline::dispatch(scope, target, event, &event_type);
    super::on_online::dispatch(scope, target, event, &event_type);
    super::on_page_hide::dispatch(scope, target, event, &event_type);
    super::on_page_reveal::dispatch(scope, target, event, &event_type);
    super::on_page_show::dispatch(scope, target, event, &event_type);
    super::on_page_swap::dispatch(scope, target, event, &event_type);
    super::on_pop_state::dispatch(scope, target, event, &event_type);
    super::on_rejection_handled::dispatch(scope, target, event, &event_type);
    super::on_storage::dispatch(scope, target, event, &event_type);
    super::on_unhandled_rejection::dispatch(scope, target, event, &event_type);
    super::on_unload::dispatch(scope, target, event, &event_type);
}

pub(crate) fn propagation_path(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    composed: bool,
) -> Vec<v8::Global<v8::Object>> {
    let mut path = vec![v8::Global::new(scope, target)];
    let mut seen = HashSet::from([target.get_identity_hash().get()]);
    let mut current = target;
    loop {
        let next = super::node::parent(scope, current)
            .or_else(|| {
                composed
                    .then(|| super::shadow_root::host(scope, current))
                    .flatten()
            })
            .or_else(|| {
                if super::document::serialize_if_document(scope, current).is_some() {
                    Some(scope.get_current_context().global(scope))
                } else {
                    None
                }
            });
        let Some(next) = next else {
            break;
        };
        let identity = next.get_identity_hash().get();
        if !seen.insert(identity) {
            break;
        }
        path.push(v8::Global::new(scope, next));
        current = next;
    }
    path
}

pub(crate) fn retargeted_target<'s>(
    scope: &v8::PinScope<'s, '_>,
    path: &[v8::Global<v8::Object>],
    current_index: usize,
) -> v8::Local<'s, v8::Object> {
    let mut exposed = v8::Local::new(scope, &path[0]);
    for boundary_index in 1..current_index {
        let boundary = v8::Local::new(scope, &path[boundary_index]);
        if let Some(host) = super::shadow_root::host(scope, boundary) {
            exposed = host;
        }
    }
    exposed
}

pub(crate) fn invoke_listeners(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
    capture: bool,
) {
    let target_id = target_record_id(scope, target);
    let mut invocations = scope
        .get_slot::<EventTargetStore>()
        .and_then(|store| store.targets.get(&target_id))
        .and_then(|record| record.listeners.get(event_type))
        .map(|listeners| {
            listeners
                .iter()
                .filter(|listener| listener.capture == capture)
                .cloned()
                .map(|listener| (listener.registration_id, Invocation::Listener(listener)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !capture
        && let Some(registration_id) = scope
            .get_slot::<EventTargetStore>()
            .and_then(|store| store.targets.get(&target_id))
            .and_then(|record| record.attribute_handlers.get(event_type))
            .copied()
    {
        invocations.push((registration_id, Invocation::Attribute(registration_id)));
    }
    invocations.sort_by_key(|(registration_id, _)| *registration_id);
    for (_, invocation) in invocations {
        let Invocation::Listener(listener) = invocation else {
            let Invocation::Attribute(registration_id) = invocation else {
                unreachable!()
            };
            let present = scope
                .get_slot::<EventTargetStore>()
                .and_then(|store| store.targets.get(&target_id))
                .and_then(|record| record.attribute_handlers.get(event_type))
                .is_some_and(|current| *current == registration_id);
            if present {
                invoke_attribute_handlers(scope, target, event, event_type);
            }
            if super::event::immediate_propagation_stopped(scope, event) {
                break;
            }
            continue;
        };
        let present = scope
            .get_slot::<EventTargetStore>()
            .and_then(|store| store.targets.get(&target_id))
            .and_then(|record| record.listeners.get(event_type))
            .is_some_and(|listeners| {
                listeners
                    .iter()
                    .any(|current| current.registration_id == listener.registration_id)
            });
        if !present {
            continue;
        }
        if listener.once {
            remove_registration(scope, target_id, event_type, listener.registration_id);
        }
        super::event::set_passive_listener(scope, event, listener.passive);
        invoke_callback(scope, target, event, &listener.callback);
        super::event::set_passive_listener(scope, event, false);
        if super::event::immediate_propagation_stopped(scope, event) {
            break;
        }
    }
}

pub(crate) fn set_attribute_handler(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event_type: &str,
    present: bool,
) {
    let target_id = target_record_id(scope, target);
    let Some(store) = scope.get_slot_mut::<EventTargetStore>() else {
        return;
    };
    if !store.targets.contains_key(&target_id) {
        return;
    }
    if !present {
        if let Some(record) = store.targets.get_mut(&target_id) {
            record.attribute_handlers.remove(event_type);
        }
        return;
    }
    if store
        .targets
        .get(&target_id)
        .is_some_and(|record| record.attribute_handlers.contains_key(event_type))
    {
        return;
    }
    let registration_id = store.next_listener_id;
    store.next_listener_id = store.next_listener_id.wrapping_add(1).max(1);
    if let Some(record) = store.targets.get_mut(&target_id) {
        record
            .attribute_handlers
            .insert(event_type.to_owned(), registration_id);
    }
}

pub(crate) fn invoke_callback(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    callback: &v8::Global<v8::Object>,
) {
    let _user_execution = crate::trace::enter_user_execution(scope);
    let callback = v8::Local::new(scope, callback);
    if let Ok(function) = v8::Local::<v8::Function>::try_from(callback) {
        v8::tc_scope!(let try_catch, scope);
        let _ = function.call(try_catch, target.into(), &[event.into()]);
        return;
    }
    let Some(key) = v8::String::new(scope, "handleEvent") else {
        return;
    };
    let Some(method) = callback.get(scope, key.into()) else {
        return;
    };
    let Ok(function) = v8::Local::<v8::Function>::try_from(method) else {
        return;
    };
    v8::tc_scope!(let try_catch, scope);
    let _ = function.call(try_catch, callback.into(), &[event.into()]);
}

pub(crate) fn remove_registration(
    scope: &mut v8::PinScope<'_, '_>,
    target_id: i32,
    event_type: &str,
    registration_id: u64,
) {
    if let Some(listeners) = scope
        .get_slot_mut::<EventTargetStore>()
        .and_then(|store| store.targets.get_mut(&target_id))
        .and_then(|record| record.listeners.get_mut(event_type))
    {
        listeners.retain(|listener| listener.registration_id != registration_id);
    }
}

pub(crate) fn resolve_waiters(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    let target_id = target_record_id(scope, target);
    let waiters = scope
        .get_slot_mut::<EventTargetStore>()
        .and_then(|store| store.targets.get_mut(&target_id))
        .and_then(|record| record.waiters.remove(event_type))
        .unwrap_or_default();
    for waiter in waiters {
        let waiter = v8::Local::new(scope, &waiter);
        let _ = waiter.resolve(scope, event.into());
    }
}

pub(crate) fn throw_invalid_state(scope: &mut v8::PinScope<'_, '_>) {
    let message = "Failed to execute 'dispatchEvent' on 'EventTarget': The event is uninitialized or is already being dispatched.";
    match super::dom_exception::create(scope, message.to_owned(), "InvalidStateError".to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(_) => crate::webidl::throw_type_error(scope, message),
    }
}

pub(crate) fn remove_signal_listeners(
    scope: &mut v8::PinScope<'_, '_>,
    signal: v8::Local<'_, v8::Object>,
) {
    let signal_identity = signal.get_identity_hash().get();
    if let Some(store) = scope.get_slot_mut::<EventTargetStore>() {
        for target in store.targets.values_mut() {
            for listeners in target.listeners.values_mut() {
                listeners.retain(|listener| listener.signal_identity != Some(signal_identity));
            }
        }
    }
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'EventTarget': Please use the 'new' operator",
        );
        return;
    }
    let object = arguments.this();
    attach(scope, object);
    result.set(object.into());
}

pub(crate) fn listener_options(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<ListenerOptions> {
    if value.is_boolean() {
        return Some(ListenerOptions {
            capture: value.boolean_value(scope),
            ..ListenerOptions::default()
        });
    }
    if value.is_null() || value.is_undefined() {
        return Some(ListenerOptions::default());
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return Some(ListenerOptions::default());
    };
    let capture = boolean_member(scope, object, "capture");
    let once = boolean_member(scope, object, "once");
    let passive = boolean_member(scope, object, "passive");
    let signal = object_member(scope, object, "signal");
    let signal = match signal {
        None => None,
        Some(value) if value.is_undefined() => None,
        Some(value) => {
            let Ok(signal) = v8::Local::<v8::Object>::try_from(value) else {
                crate::webidl::throw_type_error(
                    scope,
                    "Failed to execute 'addEventListener' on 'EventTarget': signal is not an AbortSignal.",
                );
                return None;
            };
            let Some(record) = super::abort_signal::record(scope, signal) else {
                crate::webidl::throw_type_error(
                    scope,
                    "Failed to execute 'addEventListener' on 'EventTarget': signal is not an AbortSignal.",
                );
                return None;
            };
            if record.aborted {
                return None;
            }
            Some(signal.get_identity_hash().get())
        }
    };
    Some(ListenerOptions {
        capture,
        once,
        passive,
        signal,
    })
}

pub(crate) fn capture_option(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> bool {
    if value.is_boolean() {
        value.boolean_value(scope)
    } else {
        v8::Local::<v8::Object>::try_from(value)
            .ok()
            .is_some_and(|object| boolean_member(scope, object, "capture"))
    }
}

pub(crate) fn boolean_member(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    object_member(scope, object, name).is_some_and(|value| value.boolean_value(scope))
}

pub(crate) fn object_member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn string_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: &str,
) -> v8::Local<'s, v8::Value> {
    v8::String::new(scope, value)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into())
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<EventTargetStore>() {
        store.constructors.remove(&realm_id);
    }
}
