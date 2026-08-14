use std::collections::{HashMap, HashSet};

pub(crate) struct HostInputStore {
    first_input_realms: HashSet<i32>,
    hovered_targets: HashMap<i32, v8::Global<v8::Object>>,
    active_touches: HashMap<i32, Vec<ActiveTouch>>,
    multitouch_realms: HashSet<i32>,
    moved_touch_realms: HashSet<i32>,
    suppressed_touch_mouse_realms: HashSet<i32>,
    pen_hovered_targets: HashMap<i32, v8::Global<v8::Object>>,
    pen_pointer_ids: HashMap<i32, i32>,
    active_pens: HashMap<i32, ActivePen>,
    timing_batch: Vec<v8::Global<v8::Object>>,
    timing_batch_end: f64,
    interaction_values: HashMap<i32, i32>,
    next_touch_pointer_id: i32,
}

#[derive(Clone)]
struct ActiveTouch {
    input: crate::HostTouchInput,
    start_input: crate::HostTouchInput,
    target: v8::Global<v8::Object>,
    pointer_id: i32,
    is_primary: bool,
}

#[derive(Clone)]
struct ActivePen {
    target: v8::Global<v8::Object>,
    pointer_id: i32,
    button: i16,
    interaction_id: i32,
}

impl Default for HostInputStore {
    fn default() -> Self {
        Self {
            first_input_realms: HashSet::new(),
            hovered_targets: HashMap::new(),
            active_touches: HashMap::new(),
            multitouch_realms: HashSet::new(),
            moved_touch_realms: HashSet::new(),
            suppressed_touch_mouse_realms: HashSet::new(),
            pen_hovered_targets: HashMap::new(),
            pen_pointer_ids: HashMap::new(),
            active_pens: HashMap::new(),
            timing_batch: Vec::new(),
            timing_batch_end: 0.0,
            interaction_values: HashMap::new(),
            next_touch_pointer_id: 2,
        }
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HostInputStore::default());
}

pub(crate) fn dispatch_trusted_click(
    scope: &mut v8::PinScope<'_, '_>,
    input: &crate::HostClickInput,
) -> Result<bool, String> {
    input.validate()?;
    let Some(document) = super::document_global::current(scope) else {
        return Err("host click requires an active Document".to_owned());
    };
    let Some(target) = super::document_method_support::hit_test_elements(
        scope,
        document,
        input.client_x,
        input.client_y,
    )
    .into_iter()
    .next() else {
        return Ok(false);
    };
    super::user_activation::activate_current_realm(scope);
    begin_timing_batch(scope);
    dispatch_hover_transition(scope, target, input)?;
    let interaction_id = next_interaction_id(scope);
    let pressed_buttons = button_mask(input.button);

    dispatch_pointer(
        scope,
        target,
        input,
        "pointerdown",
        pressed_buttons,
        0.5,
        interaction_id,
        false,
    )?;
    let mousedown_allowed =
        dispatch_mouse(scope, target, input, "mousedown", pressed_buttons, 0, false)?;
    if mousedown_allowed && super::html_element::is_programmatically_focusable(scope, target) {
        let focus_target = v8::Global::new(scope, target);
        super::html_element::focus_with_events(scope, focus_target)?;
    }
    dispatch_pointer(
        scope,
        target,
        input,
        "pointerup",
        0,
        0.0,
        interaction_id,
        false,
    )?;
    dispatch_mouse(scope, target, input, "mouseup", 0, 0, false)?;
    dispatch_mouse_activation(scope, target, input, interaction_id)?;
    finish_timing_batch(scope);
    Ok(true)
}

pub(crate) fn dispatch_trusted_keyboard(
    scope: &mut v8::PinScope<'_, '_>,
    input: &crate::HostKeyboardInput,
) -> Result<bool, String> {
    input.validate()?;
    let Some(target) = active_element_target(scope) else {
        return Ok(false);
    };
    super::user_activation::activate_current_realm(scope);
    begin_timing_batch(scope);
    let interaction_id = next_interaction_id(scope);
    let key_code = legacy_key_code(&input.key, &input.code);
    let keydown = create_keyboard_event(scope, input, "keydown", 0, key_code, key_code)?;
    let keydown_allowed = dispatch_trusted_event(
        scope,
        target,
        keydown,
        "keydown",
        Some(target.into()),
        interaction_id,
        false,
        true,
    )?;

    let activate_on_keyup = input.key == " " && keyboard_activation_target(scope, target);
    if keydown_allowed {
        if input.key == "Enter" {
            let keypress = create_keyboard_event(scope, input, "keypress", 13, 13, 13)?;
            let keypress_allowed = dispatch_trusted_event(
                scope,
                target,
                keypress,
                "keypress",
                Some(target.into()),
                interaction_id,
                false,
                true,
            )?;
            if keypress_allowed {
                apply_non_text_key_default(scope, target, input, interaction_id)?;
            }
        } else if let Some(text) = input
            .text
            .as_deref()
            .filter(|value| !value.is_empty())
            .filter(|_| !input.ctrl_key && !input.alt_key && !input.meta_key)
        {
            let character = text.chars().next().map(u32::from).unwrap_or(0);
            let keypress =
                create_keyboard_event(scope, input, "keypress", character, character, character)?;
            let keypress_allowed = dispatch_trusted_event(
                scope,
                target,
                keypress,
                "keypress",
                Some(target.into()),
                interaction_id,
                false,
                true,
            )?;
            if keypress_allowed && !activate_on_keyup {
                dispatch_text_input(scope, target, text, interaction_id)?;
            }
        } else {
            apply_non_text_key_default(scope, target, input, interaction_id)?;
        }
    }

    let keyup_target = active_element_target(scope).unwrap_or(target);
    let keyup = create_keyboard_event(scope, input, "keyup", 0, key_code, key_code)?;
    dispatch_trusted_event(
        scope,
        keyup_target,
        keyup,
        "keyup",
        Some(target.into()),
        interaction_id,
        true,
        true,
    )?;
    if keydown_allowed && activate_on_keyup {
        super::html_element_click::activate_trusted(scope, keyup_target)?;
    }
    finish_timing_batch(scope);
    Ok(true)
}

pub(crate) fn dispatch_trusted_wheel(
    scope: &mut v8::PinScope<'_, '_>,
    input: &crate::HostWheelInput,
) -> Result<bool, String> {
    input.validate()?;
    let Some(document) = super::document_global::current(scope) else {
        return Err("host wheel requires an active Document".to_owned());
    };
    let Some(target) = super::document_method_support::hit_test_elements(
        scope,
        document,
        input.client_x,
        input.client_y,
    )
    .into_iter()
    .next() else {
        return Ok(false);
    };
    let cancelable = super::event_target::has_non_passive_listener_on_path(scope, target, "wheel");
    let scroll_target = scrollable_ancestor(scope, target, input.delta_x, input.delta_y);
    if !cancelable {
        apply_wheel_scroll(scope, scroll_target, input);
    }
    let screen = &crate::fingerprint::edge(scope).screen;
    let view: v8::Local<v8::Value> = scope.get_current_context().global(scope).into();
    let mouse = super::mouse_event::MouseEventData {
        event_type: "wheel".to_owned(),
        screen_x: (screen.screen_x + input.client_x) as i32,
        screen_y: (screen.screen_y + input.client_y) as i32,
        client_x: input.client_x as i32,
        client_y: input.client_y as i32,
        ctrl_key: input.ctrl_key,
        shift_key: input.shift_key,
        alt_key: input.alt_key,
        meta_key: input.meta_key,
        button: 0,
        buttons: 0,
        related_target: None,
        movement_x: 0,
        movement_y: 0,
        bubbles: true,
        cancelable,
        composed: true,
        view: Some(v8::Global::new(scope, view)),
        detail: 0,
    };
    let wheel_delta_x = physical_wheel_tick(input.delta_x);
    let wheel_delta_y = physical_wheel_tick(input.delta_y);
    let event = super::wheel_event::create_with_data(
        scope,
        mouse,
        input.delta_x,
        input.delta_y,
        input.delta_z,
        input.delta_mode,
        wheel_delta_x,
        wheel_delta_y,
    )?;
    let allowed = dispatch_trusted_event(
        scope,
        target,
        event,
        "wheel",
        Some(target.into()),
        0,
        false,
        false,
    )?;
    if cancelable && allowed {
        apply_wheel_scroll(scope, scroll_target, input);
    }
    if allowed {
        if let Some(scroll_target) = scroll_target {
            let event = super::event::create(scope, "scroll")?;
            super::event::set_trusted(scope, event, true);
            super::event_target::dispatch(scope, scroll_target, event);
        }
    }
    Ok(true)
}

pub(crate) fn dispatch_trusted_drag(
    scope: &mut v8::PinScope<'_, '_>,
    input: &crate::HostDragInput,
) -> Result<bool, String> {
    input.validate()?;
    let Some(document) = super::document_global::current(scope) else {
        return Err("host drag requires an active Document".to_owned());
    };
    let start = &input.points[0];
    let Some(hit) = super::document_method_support::hit_test_elements(
        scope,
        document,
        start.client_x,
        start.client_y,
    )
    .into_iter()
    .next() else {
        return Ok(false);
    };
    let Some(source) = draggable_ancestor(scope, hit) else {
        return Ok(false);
    };
    super::user_activation::activate_current_realm(scope);
    let start_input = drag_click_input(input, start, 0);
    begin_timing_batch(scope);
    dispatch_hover_transition(scope, source, &start_input)?;
    let interaction_id = next_interaction_id(scope);
    dispatch_pointer(
        scope,
        source,
        &start_input,
        "pointerdown",
        1,
        0.5,
        interaction_id,
        false,
    )?;
    let mousedown_allowed = dispatch_mouse(scope, source, &start_input, "mousedown", 1, 0, false)?;
    if mousedown_allowed && super::html_element::is_programmatically_focusable(scope, source) {
        super::html_element::focus_with_events(scope, v8::Global::new(scope, source))?;
    }
    let first_move = &input.points[1];
    let move_input = drag_click_input(input, first_move, -1);
    let pointer_target = super::document_method_support::hit_test_elements(
        scope,
        document,
        first_move.client_x,
        first_move.client_y,
    )
    .into_iter()
    .next()
    .unwrap_or(source);
    dispatch_hover_transition_with_pointer_state(scope, pointer_target, &move_input, 1, 0.5)?;

    let transfer = super::data_transfer::create_for_drag(scope)?;
    let transfer = v8::Global::new(scope, transfer);
    let transfer_local = v8::Local::new(scope, &transfer);
    super::data_transfer::set_access_mode(
        scope,
        transfer_local,
        super::data_transfer::AccessMode::ReadWrite,
    );
    super::data_transfer::set_target_view(scope, transfer_local, false);
    let dragstart_allowed =
        dispatch_drag(scope, source, &start_input, "dragstart", 1, &transfer, true)?;
    let transfer_local = v8::Local::new(scope, &transfer);
    super::data_transfer::set_access_mode(
        scope,
        transfer_local,
        super::data_transfer::AccessMode::Protected,
    );
    if !dragstart_allowed {
        dispatch_pointer(
            scope,
            source,
            &start_input,
            "pointerup",
            0,
            0.0,
            interaction_id,
            false,
        )?;
        dispatch_mouse(scope, source, &start_input, "mouseup", 0, 0, false)?;
        finish_timing_batch(scope);
        return Ok(true);
    }
    let cancel_input = drag_click_input(
        input,
        &crate::HostDragPoint {
            client_x: 0.0,
            client_y: 0.0,
        },
        0,
    );
    dispatch_pointer_cancel(scope, pointer_target, &cancel_input, interaction_id)?;
    dispatch_pointer_transition(
        scope,
        pointer_target,
        &cancel_input,
        "pointerout",
        None,
        true,
        true,
        0,
        0.0,
    )?;
    dispatch_pointer_transition(
        scope,
        pointer_target,
        &cancel_input,
        "pointerleave",
        None,
        false,
        true,
        0,
        0.0,
    )?;
    clear_hover_target(scope);

    let mut current_target: Option<v8::Global<v8::Object>> = None;
    let mut drop_allowed = false;
    let mut final_input = start_input.clone();
    for point in input.points.iter().skip(1) {
        final_input = drag_click_input(input, point, 0);
        let transfer_local = v8::Local::new(scope, &transfer);
        super::data_transfer::set_drag_effects(
            scope,
            transfer_local,
            Some("none"),
            Some("uninitialized"),
        );
        super::data_transfer::set_access_mode(
            scope,
            transfer_local,
            super::data_transfer::AccessMode::Protected,
        );
        super::data_transfer::set_target_view(scope, transfer_local, false);
        dispatch_drag(scope, source, &final_input, "drag", 0, &transfer, true)?;
        let next_target = super::document_method_support::hit_test_elements(
            scope,
            document,
            point.client_x,
            point.client_y,
        )
        .into_iter()
        .next();
        let changed = match (&current_target, next_target) {
            (None, Some(_)) | (Some(_), None) => true,
            (Some(current), Some(next)) => {
                v8::Local::new(scope, current).get_identity_hash() != next.get_identity_hash()
            }
            (None, None) => false,
        };
        if changed {
            if let Some(next) = next_target {
                let transfer_local = v8::Local::new(scope, &transfer);
                super::data_transfer::set_drag_effects(
                    scope,
                    transfer_local,
                    Some("copy"),
                    Some("all"),
                );
                super::data_transfer::set_access_mode(
                    scope,
                    transfer_local,
                    super::data_transfer::AccessMode::Protected,
                );
                super::data_transfer::set_target_view(scope, transfer_local, true);
                dispatch_drag(scope, next, &final_input, "dragenter", 0, &transfer, true)?;
            }
            if let Some(current) = current_target.take() {
                let current = v8::Local::new(scope, &current);
                let transfer_local = v8::Local::new(scope, &transfer);
                super::data_transfer::set_drag_effects(
                    scope,
                    transfer_local,
                    Some("none"),
                    Some("all"),
                );
                super::data_transfer::set_target_view(scope, transfer_local, true);
                dispatch_drag(
                    scope,
                    current,
                    &final_input,
                    "dragleave",
                    0,
                    &transfer,
                    true,
                )?;
            }
            current_target = next_target.map(|target| v8::Global::new(scope, target));
        }
        if let Some(current) = current_target.as_ref() {
            let current = v8::Local::new(scope, current);
            let transfer_local = v8::Local::new(scope, &transfer);
            super::data_transfer::set_drag_effects(
                scope,
                transfer_local,
                Some("copy"),
                Some("all"),
            );
            super::data_transfer::set_access_mode(
                scope,
                transfer_local,
                super::data_transfer::AccessMode::Protected,
            );
            super::data_transfer::set_target_view(scope, transfer_local, true);
            drop_allowed =
                !dispatch_drag(scope, current, &final_input, "dragover", 0, &transfer, true)?;
        } else {
            drop_allowed = false;
        }
    }

    let mut final_effect = "none";
    if drop_allowed && let Some(current) = current_target.as_ref() {
        let current = v8::Local::new(scope, current);
        let transfer_local = v8::Local::new(scope, &transfer);
        super::data_transfer::set_drag_effects(scope, transfer_local, Some("copy"), Some("all"));
        super::data_transfer::set_access_mode(
            scope,
            transfer_local,
            super::data_transfer::AccessMode::ReadOnly,
        );
        super::data_transfer::set_target_view(scope, transfer_local, true);
        dispatch_drag(scope, current, &final_input, "drop", 0, &transfer, true)?;
        final_effect = "copy";
    }
    let transfer_local = v8::Local::new(scope, &transfer);
    super::data_transfer::set_drag_effects(
        scope,
        transfer_local,
        Some(final_effect),
        Some("uninitialized"),
    );
    super::data_transfer::set_access_mode(
        scope,
        transfer_local,
        super::data_transfer::AccessMode::Protected,
    );
    super::data_transfer::set_target_view(scope, transfer_local, false);
    dispatch_drag(scope, source, &final_input, "dragend", 0, &transfer, false)?;
    finish_timing_batch(scope);
    Ok(true)
}

pub(crate) fn dispatch_trusted_touch(
    scope: &mut v8::PinScope<'_, '_>,
    input: &crate::HostTouchInput,
) -> Result<bool, String> {
    input.validate()?;
    let Some(document) = super::document_global::current(scope) else {
        return Err("host touch requires an active Document".to_owned());
    };
    let realm_id = crate::webidl::realm_id(scope);
    begin_timing_batch(scope);
    let result = match input.phase {
        crate::HostTouchPhase::Start => {
            let duplicate = scope
                .get_slot::<HostInputStore>()
                .and_then(|store| store.active_touches.get(&realm_id))
                .is_some_and(|touches| {
                    touches
                        .iter()
                        .any(|touch| touch.input.identifier == input.identifier)
                });
            if duplicate {
                Err(format!(
                    "host touch identifier {} is already active",
                    input.identifier
                ))
            } else {
                let target = super::document_method_support::hit_test_elements(
                    scope,
                    document,
                    input.client_x,
                    input.client_y,
                )
                .into_iter()
                .next();
                match target {
                    None => Ok(false),
                    Some(target) => dispatch_touch_start(scope, realm_id, target, input),
                }
            }
        }
        crate::HostTouchPhase::Move => dispatch_touch_move(scope, realm_id, input),
        crate::HostTouchPhase::End | crate::HostTouchPhase::Cancel => {
            dispatch_touch_finish(scope, realm_id, input)
        }
    };
    finish_timing_batch(scope);
    result
}

pub(crate) fn dispatch_trusted_pen(
    scope: &mut v8::PinScope<'_, '_>,
    input: &crate::HostPenInput,
) -> Result<bool, String> {
    input.validate()?;
    let Some(document) = super::document_global::current(scope) else {
        return Err("host pen input requires an active Document".to_owned());
    };
    let realm_id = crate::webidl::realm_id(scope);
    begin_timing_batch(scope);
    let result = match input.phase {
        crate::HostPenPhase::Hover => {
            let target = super::document_method_support::hit_test_elements(
                scope,
                document,
                input.client_x,
                input.client_y,
            )
            .into_iter()
            .next();
            match target {
                Some(target) => {
                    dispatch_pen_hover(scope, realm_id, target, input)?;
                    Ok(true)
                }
                None => Ok(false),
            }
        }
        crate::HostPenPhase::Down => dispatch_pen_down(scope, document, realm_id, input),
        crate::HostPenPhase::Move => dispatch_pen_move(scope, realm_id, input),
        crate::HostPenPhase::Up | crate::HostPenPhase::Cancel => {
            dispatch_pen_finish(scope, realm_id, input)
        }
    };
    finish_timing_batch(scope);
    result
}

fn dispatch_pen_down(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    realm_id: i32,
    input: &crate::HostPenInput,
) -> Result<bool, String> {
    if scope
        .get_slot::<HostInputStore>()
        .is_some_and(|store| store.active_pens.contains_key(&realm_id))
    {
        return Err("host pen is already down".to_owned());
    }
    let Some(target) = super::document_method_support::hit_test_elements(
        scope,
        document,
        input.client_x,
        input.client_y,
    )
    .into_iter()
    .next() else {
        return Ok(false);
    };
    let hovered = scope
        .get_slot::<HostInputStore>()
        .and_then(|store| store.pen_hovered_targets.get(&realm_id))
        .map(|value| v8::Local::new(scope, value));
    if hovered.is_none_or(|hovered| hovered != target) {
        dispatch_pen_hover(scope, realm_id, target, input)?;
    }
    let pointer_id = pen_pointer_id(scope, realm_id)?;
    let interaction_id = next_interaction_id(scope);
    dispatch_pen_pointer(
        scope,
        target,
        input,
        "pointerdown",
        pointer_id,
        input.button,
        pen_button_mask(input.button),
        input.pressure as f32,
        true,
        true,
        true,
        None,
        interaction_id,
        false,
    )?;
    let mouse_input = pen_mouse_input(input, input.button);
    let capabilities = super::input_device_capabilities::create(scope, false)?;
    let capabilities = v8::Global::new(scope, capabilities);
    let allowed = dispatch_touch_mouse_event(
        scope,
        target,
        &mouse_input,
        "mousedown",
        pen_button_mask(input.button),
        true,
        true,
        true,
        1,
        &capabilities,
    )?;
    if allowed && super::html_element::is_programmatically_focusable(scope, target) {
        super::html_element::focus_with_events(scope, v8::Global::new(scope, target))?;
    }
    let target_global = v8::Global::new(scope, target);
    scope
        .get_slot_mut::<HostInputStore>()
        .expect("host input state")
        .active_pens
        .insert(
            realm_id,
            ActivePen {
                target: target_global,
                pointer_id,
                button: input.button,
                interaction_id,
            },
        );
    Ok(true)
}

fn dispatch_pen_move(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    input: &crate::HostPenInput,
) -> Result<bool, String> {
    let active = scope
        .get_slot::<HostInputStore>()
        .and_then(|store| store.active_pens.get(&realm_id))
        .cloned()
        .ok_or_else(|| "host pen is not down".to_owned())?;
    let target = v8::Local::new(scope, &active.target);
    dispatch_pen_pointer(
        scope,
        target,
        input,
        "pointermove",
        active.pointer_id,
        -1,
        pen_button_mask(active.button),
        input.pressure as f32,
        true,
        true,
        true,
        None,
        0,
        false,
    )?;
    let mouse_input = pen_mouse_input(input, 0);
    let capabilities = super::input_device_capabilities::create(scope, false)?;
    let capabilities = v8::Global::new(scope, capabilities);
    dispatch_touch_mouse_event(
        scope,
        target,
        &mouse_input,
        "mousemove",
        pen_button_mask(active.button),
        true,
        true,
        true,
        0,
        &capabilities,
    )?;
    Ok(true)
}

fn dispatch_pen_finish(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    input: &crate::HostPenInput,
) -> Result<bool, String> {
    let active = scope
        .get_slot_mut::<HostInputStore>()
        .and_then(|store| store.active_pens.remove(&realm_id))
        .ok_or_else(|| "host pen is not down".to_owned())?;
    let target = v8::Local::new(scope, &active.target);
    let cancel = input.phase == crate::HostPenPhase::Cancel;
    if !cancel {
        super::user_activation::activate_current_realm(scope);
    }
    dispatch_pen_pointer(
        scope,
        target,
        input,
        if cancel { "pointercancel" } else { "pointerup" },
        active.pointer_id,
        if cancel { -1 } else { active.button },
        0,
        0.0,
        true,
        !cancel,
        true,
        None,
        active.interaction_id,
        cancel,
    )?;
    if cancel {
        return Ok(true);
    }
    let mouse_input = pen_mouse_input(input, active.button);
    let capabilities = super::input_device_capabilities::create(scope, false)?;
    let capabilities = v8::Global::new(scope, capabilities);
    dispatch_touch_mouse_event(
        scope,
        target,
        &mouse_input,
        "mouseup",
        0,
        true,
        true,
        true,
        1,
        &capabilities,
    )?;
    dispatch_pen_click(
        scope,
        target,
        &mouse_input,
        active.pointer_id,
        active.interaction_id,
        &capabilities,
    )?;
    Ok(true)
}

fn dispatch_pen_hover(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostPenInput,
) -> Result<(), String> {
    let pointer_id = pen_pointer_id(scope, realm_id)?;
    let previous = scope
        .get_slot::<HostInputStore>()
        .and_then(|store| store.pen_hovered_targets.get(&realm_id))
        .map(|value| v8::Local::new(scope, value));
    let changed = previous.is_none_or(|previous| previous != target);
    let capabilities = super::input_device_capabilities::create(scope, false)?;
    let capabilities = v8::Global::new(scope, capabilities);
    let mouse_input = pen_mouse_input(input, 0);
    if changed {
        let new_path = event_path(scope, target);
        let old_path = previous
            .map(|value| event_path(scope, value))
            .unwrap_or_default();
        let common = common_root_count(&old_path, &new_path);
        if let Some(old) = previous {
            dispatch_pen_pointer(
                scope,
                old,
                input,
                "pointerout",
                pointer_id,
                -1,
                0,
                0.0,
                true,
                true,
                true,
                Some(target),
                0,
                false,
            )?;
            for node in old_path.iter().take(old_path.len().saturating_sub(common)) {
                dispatch_pen_pointer(
                    scope,
                    *node,
                    input,
                    "pointerleave",
                    pointer_id,
                    -1,
                    0,
                    0.0,
                    false,
                    false,
                    false,
                    Some(target),
                    0,
                    false,
                )?;
            }
            dispatch_touch_mouse_event(
                scope,
                old,
                &mouse_input,
                "mouseout",
                0,
                true,
                true,
                true,
                0,
                &capabilities,
            )?;
            for node in old_path.iter().take(old_path.len().saturating_sub(common)) {
                dispatch_touch_mouse_event(
                    scope,
                    *node,
                    &mouse_input,
                    "mouseleave",
                    0,
                    false,
                    false,
                    false,
                    0,
                    &capabilities,
                )?;
            }
        }
        dispatch_pen_pointer(
            scope,
            target,
            input,
            "pointerover",
            pointer_id,
            -1,
            0,
            input.pressure as f32,
            true,
            true,
            true,
            previous,
            0,
            false,
        )?;
        for node in new_path
            .iter()
            .take(new_path.len().saturating_sub(common))
            .rev()
        {
            dispatch_pen_pointer(
                scope,
                *node,
                input,
                "pointerenter",
                pointer_id,
                -1,
                0,
                input.pressure as f32,
                false,
                false,
                false,
                previous,
                0,
                false,
            )?;
        }
        dispatch_touch_mouse_event(
            scope,
            target,
            &mouse_input,
            "mouseover",
            0,
            true,
            true,
            true,
            0,
            &capabilities,
        )?;
        for node in new_path
            .iter()
            .take(new_path.len().saturating_sub(common))
            .rev()
        {
            dispatch_touch_mouse_event(
                scope,
                *node,
                &mouse_input,
                "mouseenter",
                0,
                false,
                false,
                false,
                0,
                &capabilities,
            )?;
        }
        let target_global = v8::Global::new(scope, target);
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .pen_hovered_targets
            .insert(realm_id, target_global);
    }
    dispatch_pen_pointer(
        scope,
        target,
        input,
        "pointermove",
        pointer_id,
        -1,
        0,
        input.pressure as f32,
        true,
        true,
        true,
        None,
        0,
        false,
    )?;
    dispatch_touch_mouse_event(
        scope,
        target,
        &mouse_input,
        "mousemove",
        0,
        true,
        true,
        true,
        0,
        &capabilities,
    )?;
    Ok(())
}

fn pen_pointer_id(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) -> Result<i32, String> {
    if let Some(pointer_id) = scope
        .get_slot::<HostInputStore>()
        .and_then(|store| store.pen_pointer_ids.get(&realm_id))
        .copied()
    {
        return Ok(pointer_id);
    }
    let store = scope
        .get_slot_mut::<HostInputStore>()
        .ok_or_else(|| "host input state was not prepared".to_owned())?;
    let pointer_id = store.next_touch_pointer_id;
    store.next_touch_pointer_id = store.next_touch_pointer_id.saturating_add(1).max(2);
    store.pen_pointer_ids.insert(realm_id, pointer_id);
    Ok(pointer_id)
}

fn dispatch_touch_start(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostTouchInput,
) -> Result<bool, String> {
    let (pointer_id, is_primary, was_active) = {
        let store = scope
            .get_slot_mut::<HostInputStore>()
            .ok_or_else(|| "host input state was not prepared".to_owned())?;
        let was_active = store
            .active_touches
            .get(&realm_id)
            .is_some_and(|touches| !touches.is_empty());
        let pointer_id = store.next_touch_pointer_id;
        store.next_touch_pointer_id = store.next_touch_pointer_id.saturating_add(1).max(2);
        (pointer_id, !was_active, was_active)
    };
    if was_active {
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .multitouch_realms
            .insert(realm_id);
    }
    let active = ActiveTouch {
        input: input.clone(),
        start_input: input.clone(),
        target: v8::Global::new(scope, target),
        pointer_id,
        is_primary,
    };
    scope
        .get_slot_mut::<HostInputStore>()
        .expect("host input state")
        .active_touches
        .entry(realm_id)
        .or_default()
        .push(active);

    dispatch_touch_pointer(
        scope,
        target,
        input,
        "pointerover",
        pointer_id,
        is_primary,
        1,
        input.force as f32,
        input.radius_x * 2.0,
        input.radius_y * 2.0,
        0,
        true,
        true,
        true,
        0,
        false,
    )?;
    for node in event_path(scope, target).into_iter().rev() {
        dispatch_touch_pointer(
            scope,
            node,
            input,
            "pointerenter",
            pointer_id,
            is_primary,
            1,
            input.force as f32,
            input.radius_x * 2.0,
            input.radius_y * 2.0,
            0,
            false,
            false,
            false,
            0,
            false,
        )?;
    }
    let interaction_id = next_interaction_id(scope);
    dispatch_touch_pointer(
        scope,
        target,
        input,
        "pointerdown",
        pointer_id,
        is_primary,
        1,
        input.force as f32,
        input.radius_x * 2.0,
        input.radius_y * 2.0,
        0,
        true,
        true,
        true,
        interaction_id,
        false,
    )?;
    let allowed = dispatch_touch_event(scope, realm_id, target, input, "touchstart", true)?;
    if !allowed {
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .suppressed_touch_mouse_realms
            .insert(realm_id);
    }
    Ok(true)
}

fn dispatch_touch_move(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    input: &crate::HostTouchInput,
) -> Result<bool, String> {
    let Some((index, mut active)) = active_touch(scope, realm_id, input.identifier) else {
        return Err(format!(
            "host touch identifier {} is not active",
            input.identifier
        ));
    };
    active.input = input.clone();
    if let Some(touches) = scope
        .get_slot_mut::<HostInputStore>()
        .and_then(|store| store.active_touches.get_mut(&realm_id))
    {
        touches[index] = active.clone();
    }
    let target = v8::Local::new(scope, &active.target);
    dispatch_touch_pointer(
        scope,
        target,
        input,
        "pointermove",
        active.pointer_id,
        active.is_primary,
        1,
        input.force as f32,
        input.radius_x * 2.0,
        input.radius_y * 2.0,
        -1,
        true,
        true,
        true,
        0,
        false,
    )?;
    let moved_distance = ((input.client_x - active.start_input.client_x).powi(2)
        + (input.client_y - active.start_input.client_y).powi(2))
    .sqrt();
    if moved_distance < 16.0 {
        return Ok(true);
    }
    scope
        .get_slot_mut::<HostInputStore>()
        .expect("host input state")
        .moved_touch_realms
        .insert(realm_id);
    let allowed = dispatch_touch_event(scope, realm_id, target, input, "touchmove", true)?;
    if !allowed {
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .suppressed_touch_mouse_realms
            .insert(realm_id);
    }
    Ok(true)
}

fn dispatch_touch_finish(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    input: &crate::HostTouchInput,
) -> Result<bool, String> {
    let Some((index, mut active)) = active_touch(scope, realm_id, input.identifier) else {
        return Err(format!(
            "host touch identifier {} is not active",
            input.identifier
        ));
    };
    active.input = input.clone();
    let target = v8::Local::new(scope, &active.target);
    let cancel = input.phase == crate::HostTouchPhase::Cancel;
    if !cancel {
        super::user_activation::activate_current_realm(scope);
    }
    let pointer_event_type = if cancel { "pointercancel" } else { "pointerup" };
    dispatch_touch_pointer(
        scope,
        target,
        input,
        pointer_event_type,
        active.pointer_id,
        active.is_primary,
        0,
        0.0,
        if cancel { input.radius_x * 2.0 } else { 1.0 },
        if cancel { input.radius_y * 2.0 } else { 1.0 },
        if cancel { -1 } else { 0 },
        true,
        !cancel,
        true,
        0,
        false,
    )?;
    dispatch_touch_pointer(
        scope,
        target,
        input,
        "pointerout",
        active.pointer_id,
        active.is_primary,
        0,
        0.0,
        if cancel { input.radius_x * 2.0 } else { 1.0 },
        if cancel { input.radius_y * 2.0 } else { 1.0 },
        if cancel { -1 } else { 0 },
        true,
        true,
        true,
        0,
        false,
    )?;
    for node in event_path(scope, target) {
        dispatch_touch_pointer(
            scope,
            node,
            input,
            "pointerleave",
            active.pointer_id,
            active.is_primary,
            0,
            0.0,
            if cancel { input.radius_x * 2.0 } else { 1.0 },
            if cancel { input.radius_y * 2.0 } else { 1.0 },
            if cancel { -1 } else { 0 },
            false,
            false,
            false,
            0,
            false,
        )?;
    }
    if let Some(touches) = scope
        .get_slot_mut::<HostInputStore>()
        .and_then(|store| store.active_touches.get_mut(&realm_id))
    {
        touches.remove(index);
    }
    let moved = scope
        .get_slot::<HostInputStore>()
        .is_some_and(|store| store.moved_touch_realms.contains(&realm_id));
    if moved {
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .suppressed_touch_mouse_realms
            .insert(realm_id);
    }
    let event_type = if cancel { "touchcancel" } else { "touchend" };
    let allowed = dispatch_touch_event(
        scope,
        realm_id,
        target,
        &active.input,
        event_type,
        !moved && !cancel,
    )?;
    if !allowed || cancel {
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .suppressed_touch_mouse_realms
            .insert(realm_id);
    }
    let no_active = scope
        .get_slot::<HostInputStore>()
        .and_then(|store| store.active_touches.get(&realm_id))
        .is_none_or(Vec::is_empty);
    let multitouch = scope
        .get_slot::<HostInputStore>()
        .is_some_and(|store| store.multitouch_realms.contains(&realm_id));
    let suppressed = scope
        .get_slot::<HostInputStore>()
        .is_some_and(|store| store.suppressed_touch_mouse_realms.contains(&realm_id));
    if no_active && active.is_primary && !multitouch && !suppressed && !cancel {
        dispatch_touch_compatibility_mouse(scope, target, &active)?;
    }
    if no_active {
        let store = scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state");
        store.active_touches.remove(&realm_id);
        store.multitouch_realms.remove(&realm_id);
        store.moved_touch_realms.remove(&realm_id);
        store.suppressed_touch_mouse_realms.remove(&realm_id);
    }
    Ok(true)
}

fn active_touch(
    scope: &v8::PinScope<'_, '_>,
    realm_id: i32,
    identifier: i32,
) -> Option<(usize, ActiveTouch)> {
    scope
        .get_slot::<HostInputStore>()?
        .active_touches
        .get(&realm_id)?
        .iter()
        .enumerate()
        .find(|(_, touch)| touch.input.identifier == identifier)
        .map(|(index, touch)| (index, touch.clone()))
}

#[allow(clippy::too_many_arguments)]
fn dispatch_touch_pointer(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostTouchInput,
    event_type: &str,
    pointer_id: i32,
    is_primary: bool,
    buttons: u16,
    pressure: f32,
    width: f64,
    height: f64,
    button: i16,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    interaction_id: i32,
    interaction_completed: bool,
) -> Result<bool, String> {
    let click = crate::HostClickInput {
        client_x: input.client_x,
        client_y: input.client_y,
        button,
        ctrl_key: input.ctrl_key,
        shift_key: input.shift_key,
        alt_key: input.alt_key,
        meta_key: input.meta_key,
    };
    let mut mouse = mouse_data(scope, &click, buttons);
    mouse.bubbles = bubbles;
    mouse.cancelable = cancelable;
    mouse.composed = composed;
    mouse.detail = 0;
    let pointer = super::pointer_event::PointerRecord {
        pointer_id,
        width,
        height,
        pressure,
        tilt_x: 0,
        tilt_y: 0,
        azimuth_angle: 0.0,
        altitude_angle: std::f64::consts::FRAC_PI_2,
        tangential_pressure: 0.0,
        twist: 0,
        pointer_type: "touch".to_owned(),
        is_primary,
        persistent_device_id: 0,
    };
    let event =
        super::pointer_event::create_with_data(scope, event_type.to_owned(), mouse, pointer)?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        (bubbles || super::event_target::has_listener(scope, target, event_type))
            .then_some(target.into()),
        interaction_id,
        interaction_completed,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn dispatch_pen_pointer(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostPenInput,
    event_type: &str,
    pointer_id: i32,
    button: i16,
    buttons: u16,
    pressure: f32,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    related_target: Option<v8::Local<'_, v8::Object>>,
    interaction_id: i32,
    interaction_completed: bool,
) -> Result<bool, String> {
    let click = pen_mouse_input(input, button);
    let mut mouse = mouse_data(scope, &click, buttons);
    mouse.bubbles = bubbles;
    mouse.cancelable = cancelable;
    mouse.composed = composed;
    mouse.detail = 0;
    mouse.related_target =
        related_target.map(|target| v8::Global::new(scope, v8::Local::<v8::Value>::from(target)));
    let (azimuth_angle, altitude_angle) = pen_angles(input.tilt_x, input.tilt_y);
    let pointer = super::pointer_event::PointerRecord {
        pointer_id,
        width: input.width,
        height: input.height,
        pressure,
        tilt_x: input.tilt_x,
        tilt_y: input.tilt_y,
        azimuth_angle,
        altitude_angle,
        tangential_pressure: input.tangential_pressure as f32,
        twist: input.twist,
        pointer_type: "pen".to_owned(),
        is_primary: true,
        persistent_device_id: 0,
    };
    let event =
        super::pointer_event::create_with_data(scope, event_type.to_owned(), mouse, pointer)?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        (bubbles || super::event_target::has_listener(scope, target, event_type))
            .then_some(target.into()),
        interaction_id,
        interaction_completed,
        true,
    )
}

fn pen_mouse_input(input: &crate::HostPenInput, button: i16) -> crate::HostClickInput {
    crate::HostClickInput {
        client_x: input.client_x,
        client_y: input.client_y,
        button,
        ctrl_key: input.ctrl_key,
        shift_key: input.shift_key,
        alt_key: input.alt_key,
        meta_key: input.meta_key,
    }
}

fn pen_angles(tilt_x: i32, tilt_y: i32) -> (f64, f64) {
    if tilt_x == 0 && tilt_y == 0 {
        return (0.0, std::f64::consts::FRAC_PI_2);
    }
    let tangent_x = f64::from(tilt_x).to_radians().tan();
    let tangent_y = f64::from(tilt_y).to_radians().tan();
    let mut azimuth = tangent_y.atan2(tangent_x);
    if azimuth < 0.0 {
        azimuth += std::f64::consts::TAU;
    }
    let altitude = (1.0 / tangent_x.hypot(tangent_y)).atan();
    (azimuth, altitude)
}

fn pen_button_mask(button: i16) -> u16 {
    if button == 5 { 32 } else { button_mask(button) }
}

fn create_touch<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostTouchInput,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let screen_x = super::window_view_state::screen_x(scope) + input.client_x;
    let screen_y = super::window_view_state::screen_y(scope) + input.client_y;
    let page_x = super::window_view_state::scroll_x(scope) + input.client_x;
    let page_y = super::window_view_state::scroll_y(scope) + input.client_y;
    super::touch::create_with_data(
        scope,
        super::touch::TouchRecord {
            identifier: input.identifier,
            target: v8::Global::new(scope, target),
            screen_x,
            screen_y,
            client_x: input.client_x,
            client_y: input.client_y,
            page_x,
            page_y,
            radius_x: input.radius_x,
            radius_y: input.radius_y,
            rotation_angle: input.rotation_angle,
            force: f64::from(input.force as f32),
        },
    )
}

fn dispatch_touch_event(
    scope: &mut v8::PinScope<'_, '_>,
    realm_id: i32,
    target: v8::Local<'_, v8::Object>,
    changed: &crate::HostTouchInput,
    event_type: &str,
    cancelable: bool,
) -> Result<bool, String> {
    let active = scope
        .get_slot::<HostInputStore>()
        .and_then(|store| store.active_touches.get(&realm_id))
        .cloned()
        .unwrap_or_default();
    let target_id = target.get_identity_hash();
    let mut touches = Vec::with_capacity(active.len());
    let mut target_touches = Vec::new();
    for active_touch in active {
        let active_target = v8::Local::new(scope, &active_touch.target);
        let touch = create_touch(scope, active_target, &active_touch.input)?;
        if active_target.get_identity_hash() == target_id {
            target_touches.push(touch);
        }
        touches.push(touch);
    }
    let changed_touch = create_touch(scope, target, changed)?;
    let touches = super::touch_list::create(scope, touches)?;
    let target_touches = super::touch_list::create(scope, target_touches)?;
    let changed_touches = super::touch_list::create(scope, vec![changed_touch])?;
    let capabilities = super::input_device_capabilities::create(scope, true)?;
    let event = super::touch_event::create_with_data(
        scope,
        event_type,
        touches,
        target_touches,
        changed_touches,
        changed.alt_key,
        changed.meta_key,
        changed.ctrl_key,
        changed.shift_key,
        cancelable,
        capabilities,
    )?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        Some(target.into()),
        0,
        false,
        true,
    )
}

fn dispatch_touch_compatibility_mouse(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    touch: &ActiveTouch,
) -> Result<(), String> {
    let input = crate::HostClickInput {
        client_x: touch.start_input.client_x,
        client_y: touch.start_input.client_y,
        button: 0,
        ctrl_key: touch.start_input.ctrl_key,
        shift_key: touch.start_input.shift_key,
        alt_key: touch.start_input.alt_key,
        meta_key: touch.start_input.meta_key,
    };
    let capabilities = super::input_device_capabilities::create(scope, true)?;
    let capabilities = v8::Global::new(scope, capabilities);
    dispatch_touch_mouse_event(
        scope,
        target,
        &input,
        "mouseover",
        0,
        true,
        true,
        true,
        0,
        &capabilities,
    )?;
    for node in event_path(scope, target).into_iter().rev() {
        dispatch_touch_mouse_event(
            scope,
            node,
            &input,
            "mouseenter",
            0,
            false,
            false,
            false,
            0,
            &capabilities,
        )?;
    }
    dispatch_touch_mouse_event(
        scope,
        target,
        &input,
        "mousemove",
        0,
        true,
        true,
        true,
        0,
        &capabilities,
    )?;
    let interaction_id = next_interaction_id(scope);
    let mousedown_allowed = dispatch_touch_mouse_event(
        scope,
        target,
        &input,
        "mousedown",
        1,
        true,
        true,
        true,
        1,
        &capabilities,
    )?;
    if mousedown_allowed && super::html_element::is_programmatically_focusable(scope, target) {
        super::html_element::focus_with_events(scope, v8::Global::new(scope, target))?;
    }
    dispatch_touch_mouse_event(
        scope,
        target,
        &input,
        "mouseup",
        0,
        true,
        true,
        true,
        1,
        &capabilities,
    )?;
    dispatch_touch_click(
        scope,
        target,
        &input,
        touch.pointer_id,
        interaction_id,
        &capabilities,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn dispatch_touch_mouse_event(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    event_type: &str,
    buttons: u16,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
    detail: i32,
    capabilities: &v8::Global<v8::Object>,
) -> Result<bool, String> {
    let mut data = mouse_data(scope, input, buttons);
    data.bubbles = bubbles;
    data.cancelable = cancelable;
    data.composed = composed;
    data.detail = detail;
    let event = super::mouse_event::create_with_data(scope, event_type.to_owned(), data)?;
    let capabilities_local = v8::Local::new(scope, capabilities);
    super::ui_event::set_source_capabilities(scope, event, Some(capabilities_local));
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        (bubbles || super::event_target::has_listener(scope, target, event_type))
            .then_some(target.into()),
        0,
        false,
        true,
    )
}

fn dispatch_touch_click(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    pointer_id: i32,
    interaction_id: i32,
    capabilities: &v8::Global<v8::Object>,
) -> Result<bool, String> {
    let Some(activation) = super::html_element_click::begin_trusted_activation(scope, target)
    else {
        return Ok(false);
    };
    let data = mouse_data(scope, input, 0);
    let event = super::pointer_event::create_with_data(
        scope,
        "click".to_owned(),
        data,
        super::pointer_event::PointerRecord {
            pointer_id,
            width: 1.0,
            height: 1.0,
            pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            azimuth_angle: 0.0,
            altitude_angle: std::f64::consts::FRAC_PI_2,
            tangential_pressure: 0.0,
            twist: 0,
            pointer_type: "touch".to_owned(),
            is_primary: false,
            persistent_device_id: 0,
        },
    )?;
    let capabilities_local = v8::Local::new(scope, capabilities);
    super::ui_event::set_source_capabilities(scope, event, Some(capabilities_local));
    let allowed = dispatch_trusted_event(
        scope,
        target,
        event,
        "click",
        Some(target.into()),
        interaction_id,
        true,
        true,
    )?;
    super::html_element_click::finish_trusted_activation(scope, target, activation, allowed);
    Ok(allowed)
}

fn dispatch_pen_click(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    pointer_id: i32,
    interaction_id: i32,
    capabilities: &v8::Global<v8::Object>,
) -> Result<bool, String> {
    let Some(activation) = super::html_element_click::begin_trusted_activation(scope, target)
    else {
        return Ok(false);
    };
    let data = mouse_data(scope, input, 0);
    let event = super::pointer_event::create_with_data(
        scope,
        "click".to_owned(),
        data,
        super::pointer_event::PointerRecord {
            pointer_id,
            width: 1.0,
            height: 1.0,
            pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            azimuth_angle: 0.0,
            altitude_angle: std::f64::consts::FRAC_PI_2,
            tangential_pressure: 0.0,
            twist: 0,
            pointer_type: "pen".to_owned(),
            is_primary: false,
            persistent_device_id: 0,
        },
    )?;
    let capabilities_local = v8::Local::new(scope, capabilities);
    super::ui_event::set_source_capabilities(scope, event, Some(capabilities_local));
    let allowed = dispatch_trusted_event(
        scope,
        target,
        event,
        "click",
        Some(target.into()),
        interaction_id,
        true,
        true,
    )?;
    super::html_element_click::finish_trusted_activation(scope, target, activation, allowed);
    Ok(allowed)
}

fn draggable_ancestor<'s>(
    scope: &v8::PinScope<'s, '_>,
    mut target: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    loop {
        let element = super::element::record(scope, target);
        if let Some(element) = element {
            let draggable = super::element::attribute_value(scope, target, "draggable")
                .is_some_and(|value| value.eq_ignore_ascii_case("true"));
            let default_draggable = element.tag_name == "IMG"
                || element.tag_name == "A"
                    && super::element::attribute_value(scope, target, "href").is_some();
            if draggable || default_draggable {
                return Some(target);
            }
        }
        target = super::node::parent(scope, target)?;
    }
}

fn drag_click_input(
    input: &crate::HostDragInput,
    point: &crate::HostDragPoint,
    button: i16,
) -> crate::HostClickInput {
    crate::HostClickInput {
        client_x: point.client_x,
        client_y: point.client_y,
        button,
        ctrl_key: input.ctrl_key,
        shift_key: input.shift_key,
        alt_key: input.alt_key,
        meta_key: input.meta_key,
    }
}

fn dispatch_drag(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    event_type: &str,
    buttons: u16,
    transfer: &v8::Global<v8::Object>,
    cancelable: bool,
) -> Result<bool, String> {
    let mut data = mouse_data(scope, input, buttons);
    data.detail = 0;
    data.cancelable = cancelable;
    let transfer = v8::Local::new(scope, transfer);
    let event = super::drag_event::create_with_data(scope, event_type, data, transfer)?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        Some(target.into()),
        0,
        false,
        false,
    )
}

fn active_element_target<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Object>> {
    let document = super::document_global::current(scope)?;
    super::document::stored_value(scope, document, "activeElement")
        .and_then(|value| v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &value)).ok())
        .or_else(|| super::document_property_support::find_html_element(scope, document, "BODY"))
}

fn create_keyboard_event<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    input: &crate::HostKeyboardInput,
    event_type: &str,
    char_code: u32,
    key_code: u32,
    which: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    super::keyboard_event::create_with_data(
        scope,
        event_type,
        input.key.clone(),
        input.code.clone(),
        input.location,
        input.ctrl_key,
        input.shift_key,
        input.alt_key,
        input.meta_key,
        input.repeat,
        char_code,
        key_code,
        which,
    )
}

fn dispatch_text_input(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    text: &str,
    interaction_id: i32,
) -> Result<(), String> {
    let before = super::input_event::create_with_data(
        scope,
        "beforeinput",
        Some(text.to_owned()),
        "insertText",
        true,
    )?;
    let allowed = dispatch_trusted_event(
        scope,
        target,
        before,
        "beforeinput",
        Some(target.into()),
        interaction_id,
        false,
        true,
    )?;
    if !allowed || !replace_selection(scope, target, text, false) {
        return Ok(());
    }
    let event = super::input_event::create_with_data(
        scope,
        "input",
        Some(text.to_owned()),
        "insertText",
        false,
    )?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        "input",
        Some(target.into()),
        interaction_id,
        false,
        true,
    )?;
    Ok(())
}

fn apply_non_text_key_default(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostKeyboardInput,
    interaction_id: i32,
) -> Result<(), String> {
    let (replacement, input_type, backward) = match input.key.as_str() {
        "Backspace" => ("", "deleteContentBackward", true),
        "Delete" => ("", "deleteContentForward", false),
        "Enter" if super::html_text_area_element::record(scope, target).is_some() => {
            ("\n", "insertLineBreak", false)
        }
        "Tab" => {
            move_sequential_focus(scope, target, input.shift_key)?;
            return Ok(());
        }
        _ => return Ok(()),
    };
    let before =
        super::input_event::create_with_data(scope, "beforeinput", None, input_type, true)?;
    let allowed = dispatch_trusted_event(
        scope,
        target,
        before,
        "beforeinput",
        Some(target.into()),
        interaction_id,
        false,
        true,
    )?;
    if !allowed || !replace_selection(scope, target, replacement, backward) {
        return Ok(());
    }
    let event = super::input_event::create_with_data(scope, "input", None, input_type, false)?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        "input",
        Some(target.into()),
        interaction_id,
        false,
        true,
    )?;
    Ok(())
}

fn keyboard_activation_target(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> bool {
    super::html_button_element::record(scope, target).is_some()
        || super::html_input_element::record(scope, target).is_some_and(|record| {
            matches!(
                record.input_type.as_str(),
                "checkbox" | "radio" | "button" | "submit" | "reset"
            )
        })
}

fn move_sequential_focus(
    scope: &mut v8::PinScope<'_, '_>,
    current: v8::Local<'_, v8::Object>,
    reverse: bool,
) -> Result<(), String> {
    let Some(document) = super::document_global::current(scope) else {
        return Ok(());
    };
    let candidates = super::document::document_descendants(scope, document)
        .into_iter()
        .filter(|candidate| super::html_element::is_sequentially_focusable(scope, *candidate))
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(());
    }
    let index = candidates
        .iter()
        .position(|candidate| candidate.get_identity_hash() == current.get_identity_hash())
        .unwrap_or(0);
    let next_index = if reverse {
        index.checked_sub(1).unwrap_or(candidates.len() - 1)
    } else {
        (index + 1) % candidates.len()
    };
    let next = candidates[next_index];
    if next.get_identity_hash() == current.get_identity_hash() {
        return Ok(());
    }
    super::html_element::set_focused(scope, current, false);
    let current_handle = v8::Global::new(scope, current);
    let next_handle = v8::Global::new(scope, next);
    dispatch_focus(
        scope,
        current_handle.clone(),
        "blur",
        false,
        Some(next_handle.clone()),
    )?;
    dispatch_focus(
        scope,
        current_handle.clone(),
        "focusout",
        true,
        Some(next_handle.clone()),
    )?;
    super::html_element::set_focused(scope, next, true);
    select_all_on_keyboard_focus(scope, next);
    dispatch_focus(
        scope,
        next_handle.clone(),
        "focus",
        false,
        Some(current_handle.clone()),
    )?;
    dispatch_focus(scope, next_handle, "focusin", true, Some(current_handle))?;
    Ok(())
}

fn select_all_on_keyboard_focus(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = super::html_input_element::record(scope, target) {
        if super::html_input_element::supports_selection(&record.input_type) {
            let end = record.value.encode_utf16().count().min(u32::MAX as usize) as u32;
            super::html_input_element::update(scope, target, |current| {
                current.selection_start = 0;
                current.selection_end = end;
                current.selection_direction = "none".to_owned();
            });
        }
        return;
    }
    if let Some(record) = super::html_text_area_element::record(scope, target) {
        let end = record.value.encode_utf16().count().min(u32::MAX as usize) as u32;
        if let Some(current) = scope
            .get_slot_mut::<super::html_text_area_element::HtmlTextAreaElementStore>()
            .and_then(|store| store.records.get_mut(&target.get_identity_hash().get()))
        {
            current.selection_start = 0;
            current.selection_end = end;
            current.selection_direction = "none".to_owned();
        }
    }
}

fn replace_selection(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    replacement: &str,
    backward_delete: bool,
) -> bool {
    if let Some(record) = super::html_input_element::record(scope, target) {
        if record.disabled
            || record.read_only
            || !super::html_input_element::supports_selection(&record.input_type)
        {
            return false;
        }
        let (value, start, end) = replace_utf16_selection(
            &record.value,
            record.selection_start,
            record.selection_end,
            replacement,
            backward_delete,
        );
        super::html_input_element::update(scope, target, |current| {
            current.value = value;
            current.value_dirty = true;
            current.selection_start = start;
            current.selection_end = end;
            current.selection_direction = "none".to_owned();
        });
        return true;
    }
    let Some(record) = super::html_text_area_element::record(scope, target) else {
        return false;
    };
    if record.booleans.get("disabled").copied().unwrap_or(false)
        || record.booleans.get("readOnly").copied().unwrap_or(false)
    {
        return false;
    }
    let effective_value = if record.value_dirty {
        record.value.clone()
    } else {
        super::node::node_text(scope, target)
    };
    let (value, start, end) = replace_utf16_selection(
        &effective_value,
        record.selection_start,
        record.selection_end,
        replacement,
        backward_delete,
    );
    if let Some(current) = scope
        .get_slot_mut::<super::html_text_area_element::HtmlTextAreaElementStore>()
        .and_then(|store| store.records.get_mut(&target.get_identity_hash().get()))
    {
        current.value = value;
        current.value_dirty = true;
        current.selection_start = start;
        current.selection_end = end;
        current.selection_direction = "none".to_owned();
        return true;
    }
    false
}

fn replace_utf16_selection(
    value: &str,
    selection_start: u32,
    selection_end: u32,
    replacement: &str,
    backward_delete: bool,
) -> (String, u32, u32) {
    let mut units = value.encode_utf16().collect::<Vec<_>>();
    let mut start = (selection_start as usize).min(units.len());
    let mut end = (selection_end as usize).min(units.len()).max(start);
    if start == end && replacement.is_empty() {
        if backward_delete && start > 0 {
            start -= 1;
            if start > 0 && (0xDC00..=0xDFFF).contains(&units[start]) {
                start -= 1;
            }
        } else if !backward_delete && end < units.len() {
            end += 1;
            if end < units.len() && (0xDC00..=0xDFFF).contains(&units[end]) {
                end += 1;
            }
        } else {
            return (value.to_owned(), selection_start, selection_end);
        }
    }
    let replacement_units = replacement.encode_utf16().collect::<Vec<_>>();
    units.splice(start..end, replacement_units.iter().copied());
    let caret = (start + replacement_units.len()) as u32;
    (String::from_utf16_lossy(&units), caret, caret)
}

fn legacy_key_code(key: &str, code: &str) -> u32 {
    if let Some(letter) = code.strip_prefix("Key").filter(|value| value.len() == 1) {
        return letter.as_bytes()[0].to_ascii_uppercase() as u32;
    }
    if let Some(digit) = code.strip_prefix("Digit").filter(|value| value.len() == 1) {
        return digit.as_bytes()[0] as u32;
    }
    match key {
        "Backspace" => 8,
        "Tab" => 9,
        "Enter" => 13,
        "Shift" => 16,
        "Control" => 17,
        "Alt" => 18,
        "Escape" => 27,
        " " | "Spacebar" => 32,
        "PageUp" => 33,
        "PageDown" => 34,
        "End" => 35,
        "Home" => 36,
        "ArrowLeft" => 37,
        "ArrowUp" => 38,
        "ArrowRight" => 39,
        "ArrowDown" => 40,
        "Delete" => 46,
        "Meta" => 91,
        _ => 0,
    }
}

fn scrollable_ancestor<'s>(
    scope: &v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
    delta_x: f64,
    delta_y: f64,
) -> Option<v8::Local<'s, v8::Object>> {
    let mut current = Some(target);
    while let Some(candidate) = current {
        if let Some(record) = super::element::record(scope, candidate) {
            let metrics = super::element_layout::scroll_metrics(scope, candidate);
            let can_x = delta_x != 0.0
                && ((delta_x < 0.0 && record.scroll_left > 0.0)
                    || (delta_x > 0.0
                        && record.scroll_left < metrics.scroll_width - metrics.client_width));
            let can_y = delta_y != 0.0
                && ((delta_y < 0.0 && record.scroll_top > 0.0)
                    || (delta_y > 0.0
                        && record.scroll_top < metrics.scroll_height - metrics.client_height));
            if can_x || can_y {
                return Some(candidate);
            }
        }
        current = super::node::parent(scope, candidate);
    }
    None
}

fn apply_wheel_scroll(
    scope: &mut v8::PinScope<'_, '_>,
    target: Option<v8::Local<'_, v8::Object>>,
    input: &crate::HostWheelInput,
) {
    let Some(target) = target else {
        return;
    };
    let metrics = super::element_layout::scroll_metrics(scope, target);
    let scale = match input.delta_mode {
        1 => 40.0,
        2 => metrics.client_height.max(1.0),
        _ => 1.0,
    };
    let _ = super::element::set_scroll_position(
        scope,
        target,
        input.delta_x * scale,
        input.delta_y * scale,
        true,
    );
}

fn physical_wheel_tick(delta: f64) -> f64 {
    if delta == 0.0 {
        0.0
    } else {
        -120.0 * delta.signum()
    }
}

fn dispatch_hover_transition(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
) -> Result<(), String> {
    let mut hover_input = input.clone();
    hover_input.button = -1;
    dispatch_hover_transition_with_pointer_state(scope, target, &hover_input, 0, 0.0)
}

fn dispatch_hover_transition_with_pointer_state(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    buttons: u16,
    pressure: f32,
) -> Result<(), String> {
    let realm_id = crate::webidl::realm_id(scope);
    let previous = scope
        .get_slot::<HostInputStore>()
        .and_then(|store| store.hovered_targets.get(&realm_id))
        .map(|value| v8::Local::new(scope, value));
    let changed = previous.is_none_or(|value| value != target);
    if changed {
        let new_path = event_path(scope, target);
        let old_path = previous
            .map(|value| event_path(scope, value))
            .unwrap_or_default();
        let common = common_root_count(&old_path, &new_path);
        if let Some(old) = previous {
            dispatch_pointer_transition(
                scope,
                old,
                input,
                "pointerout",
                Some(target),
                true,
                true,
                buttons,
                pressure,
            )?;
            for node in old_path.iter().take(old_path.len().saturating_sub(common)) {
                dispatch_pointer_transition(
                    scope,
                    *node,
                    input,
                    "pointerleave",
                    Some(target),
                    false,
                    true,
                    buttons,
                    pressure,
                )?;
            }
        }
        dispatch_pointer_transition(
            scope,
            target,
            input,
            "pointerover",
            previous,
            true,
            true,
            buttons,
            pressure,
        )?;
        for node in new_path
            .iter()
            .take(new_path.len().saturating_sub(common))
            .rev()
        {
            dispatch_pointer_transition(
                scope,
                *node,
                input,
                "pointerenter",
                previous,
                false,
                true,
                buttons,
                pressure,
            )?;
        }
        if let Some(old) = previous {
            dispatch_mouse_transition(
                scope,
                old,
                input,
                "mouseout",
                Some(target),
                true,
                true,
                buttons,
            )?;
            for node in old_path.iter().take(old_path.len().saturating_sub(common)) {
                dispatch_mouse_transition(
                    scope,
                    *node,
                    input,
                    "mouseleave",
                    Some(target),
                    false,
                    false,
                    buttons,
                )?;
            }
        }
        dispatch_mouse_transition(
            scope,
            target,
            input,
            "mouseover",
            previous,
            true,
            true,
            buttons,
        )?;
        for node in new_path
            .iter()
            .take(new_path.len().saturating_sub(common))
            .rev()
        {
            let record_timing = super::event_target::has_listener(scope, *node, "mouseenter");
            dispatch_mouse_transition(
                scope,
                *node,
                input,
                "mouseenter",
                previous,
                false,
                record_timing,
                buttons,
            )?;
        }
        let target_global = v8::Global::new(scope, target);
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .hovered_targets
            .insert(realm_id, target_global);
    }
    dispatch_pointer_transition(
        scope,
        target,
        input,
        "pointermove",
        None,
        true,
        false,
        buttons,
        pressure,
    )?;
    dispatch_mouse_transition(
        scope,
        target,
        input,
        "mousemove",
        None,
        true,
        false,
        buttons,
    )?;
    Ok(())
}

fn clear_hover_target(scope: &mut v8::PinScope<'_, '_>) {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(store) = scope.get_slot_mut::<HostInputStore>() {
        store.hovered_targets.remove(&realm_id);
    }
}

fn event_path<'s>(
    scope: &v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    let mut path = Vec::new();
    let mut current = Some(target);
    while let Some(node) = current {
        path.push(node);
        current = super::node::parent(scope, node);
    }
    path
}

fn common_root_count(
    left: &[v8::Local<'_, v8::Object>],
    right: &[v8::Local<'_, v8::Object>],
) -> usize {
    left.iter()
        .rev()
        .zip(right.iter().rev())
        .take_while(|(left, right)| left.get_identity_hash() == right.get_identity_hash())
        .count()
}

fn dispatch_focus(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Global<v8::Object>,
    event_type: &str,
    bubbles: bool,
    related_target: Option<v8::Global<v8::Object>>,
) -> Result<(), String> {
    let event =
        super::focus_event::create_with_data(scope, event_type, bubbles, true, related_target)?;
    super::event::set_trusted(scope, event, true);
    let target = v8::Local::new(scope, &target);
    super::event_target::dispatch(scope, target, event);
    super::performance::record_host_input_event(scope, event_type, false);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn dispatch_pointer(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    event_type: &str,
    buttons: u16,
    pressure: f32,
    interaction_id: i32,
    interaction_completed: bool,
) -> Result<bool, String> {
    let mouse_data = mouse_data(scope, input, buttons);
    let pointer = super::pointer_event::PointerRecord {
        pointer_id: 1,
        width: 1.0,
        height: 1.0,
        pressure,
        tilt_x: 0,
        tilt_y: 0,
        azimuth_angle: 0.0,
        altitude_angle: std::f64::consts::FRAC_PI_2,
        tangential_pressure: 0.0,
        twist: 0,
        pointer_type: "mouse".to_owned(),
        is_primary: true,
        persistent_device_id: 0,
    };
    let event =
        super::pointer_event::create_with_data(scope, event_type.to_owned(), mouse_data, pointer)?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        Some(target.into()),
        interaction_id,
        interaction_completed,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
fn dispatch_pointer_transition(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    event_type: &str,
    related_target: Option<v8::Local<'_, v8::Object>>,
    bubbles: bool,
    record_timing: bool,
    buttons: u16,
    pressure: f32,
) -> Result<bool, String> {
    let mut mouse_data = mouse_data(scope, input, buttons);
    mouse_data.related_target =
        related_target.map(|value| v8::Global::new(scope, v8::Local::<v8::Value>::from(value)));
    mouse_data.bubbles = bubbles;
    mouse_data.cancelable = bubbles;
    mouse_data.composed = bubbles;
    mouse_data.detail = 0;
    let pointer = super::pointer_event::PointerRecord {
        pointer_id: 1,
        width: 1.0,
        height: 1.0,
        pressure,
        tilt_x: 0,
        tilt_y: 0,
        azimuth_angle: 0.0,
        altitude_angle: std::f64::consts::FRAC_PI_2,
        tangential_pressure: 0.0,
        twist: 0,
        pointer_type: "mouse".to_owned(),
        is_primary: true,
        persistent_device_id: 0,
    };
    let event =
        super::pointer_event::create_with_data(scope, event_type.to_owned(), mouse_data, pointer)?;
    let timing_target = (bubbles || super::event_target::has_listener(scope, target, event_type))
        .then_some(target.into());
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        timing_target,
        0,
        false,
        record_timing,
    )
}

fn dispatch_pointer_cancel(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    interaction_id: i32,
) -> Result<bool, String> {
    let mut mouse_data = mouse_data(scope, input, 0);
    mouse_data.cancelable = false;
    mouse_data.detail = 0;
    let pointer = super::pointer_event::PointerRecord {
        pointer_id: 1,
        width: 1.0,
        height: 1.0,
        pressure: 0.0,
        tilt_x: 0,
        tilt_y: 0,
        azimuth_angle: 0.0,
        altitude_angle: std::f64::consts::FRAC_PI_2,
        tangential_pressure: 0.0,
        twist: 0,
        pointer_type: "mouse".to_owned(),
        is_primary: true,
        persistent_device_id: 0,
    };
    let event = super::pointer_event::create_with_data(
        scope,
        "pointercancel".to_owned(),
        mouse_data,
        pointer,
    )?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        "pointercancel",
        Some(target.into()),
        interaction_id,
        true,
        true,
    )
}

fn dispatch_mouse(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    event_type: &str,
    buttons: u16,
    interaction_id: i32,
    interaction_completed: bool,
) -> Result<bool, String> {
    let data = mouse_data(scope, input, buttons);
    let event = super::mouse_event::create_with_data(scope, event_type.to_owned(), data)?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        Some(target.into()),
        interaction_id,
        interaction_completed,
        true,
    )
}

fn dispatch_mouse_activation(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    interaction_id: i32,
) -> Result<bool, String> {
    let Some(activation) = super::html_element_click::begin_trusted_activation(scope, target)
    else {
        return Ok(false);
    };
    let data = mouse_data(scope, input, 0);
    let event = super::pointer_event::create_with_data(
        scope,
        "click".to_owned(),
        data,
        super::pointer_event::PointerRecord {
            pointer_id: 1,
            width: 1.0,
            height: 1.0,
            pressure: 0.0,
            tilt_x: 0,
            tilt_y: 0,
            azimuth_angle: 0.0,
            altitude_angle: std::f64::consts::FRAC_PI_2,
            tangential_pressure: 0.0,
            twist: 0,
            pointer_type: "mouse".to_owned(),
            is_primary: true,
            persistent_device_id: 0,
        },
    )?;
    let allowed = dispatch_trusted_event(
        scope,
        target,
        event,
        "click",
        Some(target.into()),
        interaction_id,
        true,
        true,
    )?;
    super::html_element_click::finish_trusted_activation(scope, target, activation, allowed);
    Ok(allowed)
}

#[allow(clippy::too_many_arguments)]
fn dispatch_mouse_transition(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    input: &crate::HostClickInput,
    event_type: &str,
    related_target: Option<v8::Local<'_, v8::Object>>,
    bubbles: bool,
    record_timing: bool,
    buttons: u16,
) -> Result<bool, String> {
    let mut data = mouse_data(scope, input, buttons);
    data.related_target =
        related_target.map(|value| v8::Global::new(scope, v8::Local::<v8::Value>::from(value)));
    data.bubbles = bubbles;
    data.cancelable = bubbles;
    data.composed = bubbles;
    data.detail = 0;
    let event = super::mouse_event::create_with_data(scope, event_type.to_owned(), data)?;
    dispatch_trusted_event(
        scope,
        target,
        event,
        event_type,
        (bubbles || super::event_target::has_listener(scope, target, event_type))
            .then_some(target.into()),
        0,
        false,
        record_timing,
    )
}

fn mouse_data(
    scope: &mut v8::PinScope<'_, '_>,
    input: &crate::HostClickInput,
    buttons: u16,
) -> super::mouse_event::MouseEventData {
    let screen = &crate::fingerprint::edge(scope).screen;
    let view: v8::Local<v8::Value> = scope.get_current_context().global(scope).into();
    super::mouse_event::MouseEventData {
        event_type: String::new(),
        screen_x: (screen.screen_x + input.client_x) as i32,
        screen_y: (screen.screen_y + input.client_y) as i32,
        client_x: input.client_x as i32,
        client_y: input.client_y as i32,
        ctrl_key: input.ctrl_key,
        shift_key: input.shift_key,
        alt_key: input.alt_key,
        meta_key: input.meta_key,
        button: input.button,
        buttons,
        related_target: None,
        movement_x: 0,
        movement_y: 0,
        bubbles: true,
        cancelable: true,
        composed: true,
        view: Some(v8::Global::new(scope, view)),
        detail: 1,
    }
}

fn dispatch_trusted_event(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
    timing_target: Option<v8::Local<'_, v8::Value>>,
    interaction_id: i32,
    interaction_completed: bool,
    record_timing: bool,
) -> Result<bool, String> {
    let start_time = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
    super::event::set_trusted(scope, event, true);
    let allowed = super::event_target::dispatch(scope, target, event);
    let end_time = super::performance::now_for_current_realm(scope).unwrap_or(start_time);
    if record_timing {
        let duration = (((end_time - start_time).max(0.0) / 8.0).ceil() * 8.0).max(8.0);
        let cancelable = super::event::record(scope, event)
            .map(|record| record.cancelable)
            .unwrap_or(false);
        let timing = super::performance_event_timing::create_with_entry_type(
            scope,
            event_type.to_owned(),
            "event",
            start_time,
            duration,
            cancelable,
            timing_target,
            interaction_id,
        )?;
        super::performance_event_timing::set_processing_times(scope, timing, start_time, end_time);
        super::performance_observer::queue_entry(scope, timing, "event");
        track_timing_entry(scope, timing, end_time);
        if matches!(event_type, "pointerdown" | "keydown") && mark_first_input(scope) {
            let first_input = super::performance_event_timing::create_with_entry_type(
                scope,
                event_type.to_owned(),
                "first-input",
                start_time,
                duration,
                cancelable,
                timing_target,
                interaction_id,
            )?;
            super::performance_event_timing::set_processing_times(
                scope,
                first_input,
                start_time,
                end_time,
            );
            super::performance_observer::queue_entry(scope, first_input, "first-input");
            track_timing_entry(scope, first_input, end_time);
        }
    }
    super::performance::record_host_input_event(scope, event_type, interaction_completed);
    Ok(allowed)
}

fn begin_timing_batch(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<HostInputStore>() {
        store.timing_batch.clear();
        store.timing_batch_end = 0.0;
    }
}

fn track_timing_entry(
    scope: &mut v8::PinScope<'_, '_>,
    entry: v8::Local<'_, v8::Object>,
    processing_end: f64,
) {
    let entry = v8::Global::new(scope, entry);
    if let Some(store) = scope.get_slot_mut::<HostInputStore>() {
        store.timing_batch.push(entry);
        store.timing_batch_end = store.timing_batch_end.max(processing_end);
    }
}

fn finish_timing_batch(scope: &mut v8::PinScope<'_, '_>) {
    let Some((entries, processing_end)) = scope.get_slot_mut::<HostInputStore>().map(|store| {
        (
            std::mem::take(&mut store.timing_batch),
            std::mem::take(&mut store.timing_batch_end),
        )
    }) else {
        return;
    };
    let realm_id = crate::webidl::realm_id(scope);
    let now_monotonic = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let now_performance = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
    let interaction_start = entries
        .iter()
        .filter_map(|entry| {
            super::performance_entry::record(scope, v8::Local::new(scope, entry))
                .map(|base| base.start_time)
        })
        .min_by(f64::total_cmp)
        .unwrap_or(processing_end);
    let start_monotonic = now_monotonic - (now_performance - interaction_start);
    let end_monotonic = now_monotonic - (now_performance - processing_end);
    // Chromium's Event Timing duration includes presentation.  Input arriving
    // during a frame is exposed after the following compositor presentation,
    // i.e. the second 60Hz rendering opportunity after the event start.  A
    // long handler can push that opportunity forward.  Public durations are
    // quantized to 8ms buckets.
    let first_frame =
        super::animation_frame_state::next_rendering_opportunity(scope, realm_id, start_monotonic);
    let mut presentation_monotonic =
        super::animation_frame_state::next_rendering_opportunity(scope, realm_id, first_frame);
    while presentation_monotonic < end_monotonic {
        presentation_monotonic = super::animation_frame_state::next_rendering_opportunity(
            scope,
            realm_id,
            presentation_monotonic,
        );
    }
    let batch_duration =
        (((presentation_monotonic - start_monotonic).max(0.0) / 8.0).round() * 8.0).max(16.0);
    for entry in entries {
        let entry = v8::Local::new(scope, &entry);
        if super::performance_entry::record(scope, entry).is_none() {
            continue;
        }
        super::performance_entry::set_duration(scope, entry, batch_duration);
    }
}

fn next_interaction_id(scope: &mut v8::PinScope<'_, '_>) -> i32 {
    const MINIMUM_INITIAL_VALUE: u32 = 100;
    const MAXIMUM_INITIAL_VALUE: u32 = 10_000;
    const INCREMENT: i32 = 7;

    let realm_id = crate::webidl::realm_id(scope);
    let needs_initial_value = scope
        .get_slot::<HostInputStore>()
        .is_none_or(|store| !store.interaction_values.contains_key(&realm_id));
    if needs_initial_value {
        let span = MAXIMUM_INITIAL_VALUE - MINIMUM_INITIAL_VALUE + 1;
        // Blink seeds each Window's interaction counter independently with a
        // random integer in [100, 10000]. Rejection sampling avoids modulo
        // bias while still respecting the sandbox deterministic RNG option.
        let threshold = span.wrapping_neg() % span;
        let mut sampled = None;
        for _ in 0..8 {
            let mut bytes = [0_u8; 4];
            if !super::crypto::fill_random(scope, &mut bytes) {
                break;
            }
            let value = u32::from_le_bytes(bytes);
            if value >= threshold {
                sampled = Some(MINIMUM_INITIAL_VALUE + value % span);
                break;
            }
        }
        let initial = sampled.unwrap_or_else(|| {
            let time_bits = crate::determinism::monotonic_snapshot_milliseconds(scope).to_bits();
            MINIMUM_INITIAL_VALUE
                + ((time_bits as u32) ^ (time_bits >> 32) as u32 ^ realm_id as u32) % span
        }) as i32;
        scope
            .get_slot_mut::<HostInputStore>()
            .expect("host input state")
            .interaction_values
            .insert(realm_id, initial);
    }
    let store = scope
        .get_slot_mut::<HostInputStore>()
        .expect("host input state");
    let value = store
        .interaction_values
        .get_mut(&realm_id)
        .expect("interaction value");
    *value = value.saturating_add(INCREMENT);
    *value
}

fn mark_first_input(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<HostInputStore>()
        .expect("host input state")
        .first_input_realms
        .insert(realm_id)
}

fn button_mask(button: i16) -> u16 {
    match button {
        0 => 1,
        1 => 4,
        2 => 2,
        3 => 8,
        4 => 16,
        _ => 0,
    }
}
