use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AnimationFrameState {
    realms: HashMap<i32, AnimationFrameRealmState>,
}

struct AnimationFrameRealmState {
    next_id: i32,
    callbacks: HashMap<i32, AnimationFrameRecord>,
    next_due_ms: Option<f64>,
    last_frame_ms: Option<f64>,
}

struct AnimationFrameRecord {
    callback: v8::Global<v8::Function>,
    context: v8::Global<v8::Context>,
}

const FRAME_INTERVAL_MS: f64 = 1_000.0 / 60.0;

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AnimationFrameState::default());
}

pub(crate) fn next_due(scope: &v8::PinScope<'_, '_>) -> Option<f64> {
    scope.get_slot::<AnimationFrameState>().and_then(|state| {
        state
            .realms
            .values()
            .filter_map(|realm| realm.next_due_ms)
            .min_by(f64::total_cmp)
    })
}

pub(crate) fn cancel(scope: &mut v8::PinScope<'_, '_>, id: i32) {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(realm) = scope
        .get_slot_mut::<AnimationFrameState>()
        .and_then(|state| state.realms.get_mut(&realm_id))
    {
        realm.callbacks.remove(&id);
        if realm.callbacks.is_empty() {
            realm.next_due_ms = None;
        }
    }
}

pub(crate) fn reserve(
    scope: &mut v8::PinScope<'_, '_>,
    callback: v8::Local<'_, v8::Function>,
) -> i32 {
    let stored_callback = v8::Global::new(scope, callback);
    let context = v8::Global::new(scope, scope.get_current_context());
    let realm_id = crate::webidl::realm_id(scope);
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let Some(state) = scope.get_slot_mut::<AnimationFrameState>() else {
        return 0;
    };
    let realm = state
        .realms
        .entry(realm_id)
        .or_insert_with(|| AnimationFrameRealmState {
            next_id: 1,
            callbacks: HashMap::new(),
            next_due_ms: None,
            last_frame_ms: None,
        });
    // requestAnimationFrame is aligned to the document's rendering
    // opportunities.  A callback requested from inside a frame targets the
    // next vsync boundary, not `callback completion time + 16.67ms`; the
    // latter accumulates layout/callback overhead on every nested RAF.
    let due_ms = realm
        .last_frame_ms
        .map(|last_frame| rendering_opportunity_for_request(last_frame, now))
        .unwrap_or(now + FRAME_INTERVAL_MS);
    let id = realm.next_id;
    realm.next_id = realm.next_id.saturating_add(1).max(1);
    realm.callbacks.insert(
        id,
        AnimationFrameRecord {
            callback: stored_callback,
            context,
        },
    );
    realm.next_due_ms.get_or_insert(due_ms);
    id
}

/// Samples the document timeline at the start of an HTML task.
///
/// Blink exposes the most recent rendering opportunity through
/// `document.timeline.currentTime` while ordinary JavaScript in that task is
/// running.  The value is stable for the rest of the task even though
/// `performance.now()` continues to advance.  Keeping this sampling next to
/// the RAF grid also prevents timers and scheduler tasks from inventing a
/// second, drifting document clock.
pub(crate) fn sample_task_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let sample = {
        let Some(state) = scope.get_slot_mut::<AnimationFrameState>() else {
            return;
        };
        let realm = state
            .realms
            .entry(realm_id)
            .or_insert_with(|| AnimationFrameRealmState {
                next_id: 1,
                callbacks: HashMap::new(),
                next_due_ms: None,
                last_frame_ms: None,
            });
        let sample = realm
            .last_frame_ms
            .map(|last_frame| latest_rendering_opportunity_at_or_before(last_frame, now))
            .unwrap_or(now);
        realm.last_frame_ms = Some(sample);
        sample
    };
    let timestamp =
        super::performance::now_for_realm_at(scope, realm_id, sample).unwrap_or_else(|| {
            crate::determinism::relative_high_resolution_milliseconds(scope, sample, 0.0)
        });
    super::animation_timeline::sample_realm_at(scope, realm_id, timestamp);
}

pub(crate) fn sample_current_task_realm(scope: &mut v8::PinScope<'_, '_>) {
    sample_task_realm(scope, crate::webidl::realm_id(scope));
}

pub(crate) fn run_ready(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let monotonic_now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    let mut ready_realms = scope
        .get_slot::<AnimationFrameState>()
        .map(|state| {
            state
                .realms
                .iter()
                .filter(|(_, realm)| realm.next_due_ms.is_some_and(|due| due <= monotonic_now))
                .map(|(realm_id, _)| *realm_id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ready_realms.sort_unstable();

    let mut ran = false;
    for realm_id in ready_realms {
        let ready = scope
            .get_slot_mut::<AnimationFrameState>()
            .and_then(|state| state.realms.get_mut(&realm_id))
            .map(|realm| {
                let frame_time = realm.next_due_ms.unwrap_or(monotonic_now);
                realm.next_due_ms = None;
                realm.last_frame_ms = Some(frame_time);
                (frame_time, std::mem::take(&mut realm.callbacks))
            });
        let Some((frame_time, callbacks)) = ready else {
            continue;
        };
        if callbacks.is_empty() {
            continue;
        }
        // RAF's timestamp describes the rendering opportunity, not the wall
        // time at which a delayed callback finally begins to execute.
        let timestamp = super::performance::now_for_realm_at(scope, realm_id, frame_time)
            .unwrap_or_else(|| {
                crate::determinism::relative_high_resolution_milliseconds(scope, frame_time, 0.0)
            });
        let mut callbacks = callbacks.into_iter().collect::<Vec<_>>();
        callbacks.sort_by_key(|(id, _)| *id);
        let rendering_context = callbacks.first().map(|(_, record)| record.context.clone());
        for (_, record) in callbacks {
            let context = v8::Local::new(scope, &record.context);
            let callback_scope = &mut v8::ContextScope::new(scope, context);
            let task_start = super::performance_observer::task_start(callback_scope);
            super::animation_timeline::sample_realm_at(callback_scope, realm_id, timestamp);
            super::animation::sample_realm_at(callback_scope, realm_id, timestamp);
            let receiver: v8::Local<v8::Value> = callback_scope
                .get_current_context()
                .global(callback_scope)
                .into();
            let timestamp: v8::Local<v8::Value> = v8::Number::new(callback_scope, timestamp).into();
            let callback = v8::Local::new(callback_scope, &record.callback);
            let _ = callback.call(callback_scope, receiver, &[timestamp]);
            callback_scope.perform_microtask_checkpoint();
            if super::performance_observer::record_completed_task(callback_scope, task_start, true)
            {
                callback_scope.perform_microtask_checkpoint();
            }
        }
        if let Some(context) = rendering_context {
            let context = v8::Local::new(scope, &context);
            let rendering_scope = &mut v8::ContextScope::new(scope, context);
            super::rendering_performance_state::update(rendering_scope);
            rendering_scope.perform_microtask_checkpoint();
        }
        ran = true;
    }
    ran
}

pub(crate) fn next_rendering_opportunity(
    scope: &v8::PinScope<'_, '_>,
    realm_id: i32,
    now: f64,
) -> f64 {
    let last_frame = scope
        .get_slot::<AnimationFrameState>()
        .and_then(|state| state.realms.get(&realm_id))
        .and_then(|realm| realm.last_frame_ms);
    let Some(last_frame) = last_frame else {
        return now + FRAME_INTERVAL_MS;
    };
    next_rendering_opportunity_after(last_frame, now)
}

fn next_rendering_opportunity_after(last_frame: f64, now: f64) -> f64 {
    let elapsed = (now - last_frame).max(0.0);
    let intervals = (elapsed / FRAME_INTERVAL_MS).floor() + 1.0;
    let candidate = last_frame + intervals * FRAME_INTERVAL_MS;
    // Re-feeding a previously returned opportunity can make the division
    // land just below its mathematical integer because 1000/60 is not exactly
    // representable.  In that case the old formula returned the same value
    // forever.  Event Timing advances through presentation opportunities in
    // a loop, so a handler spanning more than two frames could deadlock the
    // isolated worker.  A "next" opportunity must be strictly later.
    if candidate > now {
        candidate
    } else {
        candidate + FRAME_INTERVAL_MS
    }
}

fn rendering_opportunity_for_request(last_frame: f64, now: f64) -> f64 {
    let next = last_frame + FRAME_INTERVAL_MS;
    if next <= now {
        // JavaScript can request a frame late in a long task whose document
        // timeline is still frozen at task start.  Blink associates that RAF
        // with the latest rendering opportunity already reached by the task,
        // rather than inventing a timestamp after callback execution.
        latest_rendering_opportunity_at_or_before(last_frame, now)
    } else {
        next
    }
}

fn latest_rendering_opportunity_at_or_before(last_frame: f64, now: f64) -> f64 {
    if now <= last_frame {
        return last_frame;
    }
    let elapsed = now - last_frame;
    let intervals = (elapsed / FRAME_INTERVAL_MS).floor();
    let mut candidate = last_frame + intervals * FRAME_INTERVAL_MS;
    // Floating-point multiplication can put the mathematical boundary a few
    // ulps after `now`.  A task must never observe a future document time.
    if candidate > now {
        candidate -= FRAME_INTERVAL_MS;
    }
    candidate.max(last_frame)
}

#[cfg(test)]
mod tests {
    use super::{
        FRAME_INTERVAL_MS, latest_rendering_opportunity_at_or_before,
        next_rendering_opportunity_after, rendering_opportunity_for_request,
    };

    #[test]
    fn rendering_opportunities_always_advance_across_fractional_frame_boundaries() {
        let last_frame = 83.742_361_927_032_47;
        let mut current = last_frame;
        for _ in 0..1_000 {
            let next = next_rendering_opportunity_after(last_frame, current);
            assert!(next > current);
            assert!(next - current <= FRAME_INTERVAL_MS * 1.000_000_000_001);
            current = next;
        }
    }

    #[test]
    fn nested_animation_frame_uses_the_existing_vsync_grid() {
        let last_frame = 100.0;
        let callback_time = 112.0;
        assert_eq!(
            rendering_opportunity_for_request(last_frame, callback_time),
            last_frame + FRAME_INTERVAL_MS
        );
    }

    #[test]
    fn request_late_in_a_long_task_uses_the_latest_reached_boundary() {
        let last_frame = 100.0;
        assert_eq!(
            rendering_opportunity_for_request(last_frame, 181.0),
            last_frame + FRAME_INTERVAL_MS * 4.0
        );
    }

    #[test]
    fn task_sampling_uses_the_latest_existing_vsync_boundary() {
        let last_frame = 100.0;
        assert_eq!(
            latest_rendering_opportunity_at_or_before(last_frame, 112.0),
            last_frame
        );
        assert_eq!(
            latest_rendering_opportunity_at_or_before(last_frame, 137.0),
            last_frame + FRAME_INTERVAL_MS * 2.0
        );
        let exact = last_frame + FRAME_INTERVAL_MS * 9.0;
        assert!(latest_rendering_opportunity_at_or_before(last_frame, exact) <= exact);
    }
}
