use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const EDGE_NON_ISOLATED_CLOCK_RESOLUTION_MS: f64 = 0.1;
const MAX_CLOCK_SLEEP_SLICE: Duration = Duration::from_millis(5);

pub(crate) struct DeterminismState {
    configuration: crate::DeterministicExecution,
    elapsed_ms: u64,
    started_at: Instant,
    epoch_origin_ms: f64,
    random_state: u64,
    original_date_by_global: HashMap<i32, v8::Global<v8::Function>>,
}

pub(crate) fn prepare(
    isolate: &mut v8::OwnedIsolate,
    configuration: crate::DeterministicExecution,
) {
    let random_state = configuration
        .random_seed
        .unwrap_or(0)
        .wrapping_add(0x9e37_79b9_7f4a_7c15);
    isolate.set_slot(DeterminismState {
        configuration,
        elapsed_ms: 0,
        started_at: Instant::now(),
        epoch_origin_ms: system_epoch_milliseconds(),
        random_state,
        original_date_by_global: HashMap::new(),
    });
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    if clock_is_deterministic(scope) {
        install_date(scope)?;
    }
    if random_is_deterministic(scope) {
        install_math_random(scope)?;
    }
    Ok(())
}

pub(crate) fn max_task_turns(scope: &v8::PinScope<'_, '_>) -> usize {
    scope
        .get_slot::<DeterminismState>()
        .map(|state| state.configuration.max_task_turns)
        .unwrap_or(1)
}

pub(crate) fn advance_task_turn(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(state) = scope.get_slot_mut::<DeterminismState>()
        && state.configuration.clock_epoch_ms.is_some()
    {
        state.elapsed_ms = state
            .elapsed_ms
            .saturating_add(state.configuration.clock_step_ms);
    }
}

pub(crate) fn advance_to_milliseconds(scope: &mut v8::PinScope<'_, '_>, elapsed_ms: u64) {
    if let Some(state) = scope.get_slot_mut::<DeterminismState>()
        && state.configuration.clock_epoch_ms.is_some()
    {
        state.elapsed_ms = state.elapsed_ms.max(elapsed_ms);
    }
}

pub(crate) fn epoch_milliseconds(scope: &v8::PinScope<'_, '_>) -> f64 {
    let Some(state) = scope.get_slot::<DeterminismState>() else {
        return system_epoch_milliseconds();
    };
    state
        .configuration
        .clock_epoch_ms
        .map_or(state.epoch_origin_ms, |epoch| epoch as f64)
        + elapsed_milliseconds_from_state(state)
}

pub(crate) fn elapsed_milliseconds(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<DeterminismState>()
        .map(elapsed_milliseconds_from_state)
        .unwrap_or(0.0)
}

pub(crate) fn high_resolution_milliseconds(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let value = value.max(0.0);
    (value / EDGE_NON_ISOLATED_CLOCK_RESOLUTION_MS).floor() * EDGE_NON_ISOLATED_CLOCK_RESOLUTION_MS
}

pub(crate) fn date_epoch_milliseconds(scope: &v8::PinScope<'_, '_>) -> f64 {
    epoch_milliseconds(scope).floor()
}

pub(crate) fn wait_until_elapsed(scope: &mut v8::PinScope<'_, '_>, due_ms: f64) -> bool {
    if clock_is_deterministic(scope) {
        advance_to_milliseconds(scope, due_ms.max(0.0).ceil() as u64);
        return true;
    }
    loop {
        if scope.is_execution_terminating() {
            return false;
        }
        let remaining_ms = due_ms - elapsed_milliseconds(scope);
        if remaining_ms <= 0.0 {
            return true;
        }
        std::thread::sleep(
            Duration::from_secs_f64(remaining_ms / 1_000.0).min(MAX_CLOCK_SLEEP_SLICE),
        );
    }
}

fn elapsed_milliseconds_from_state(state: &DeterminismState) -> f64 {
    if state.configuration.clock_epoch_ms.is_some() {
        state.elapsed_ms as f64
    } else {
        state.started_at.elapsed().as_secs_f64() * 1_000.0
    }
}

pub(crate) fn fill_random(scope: &mut v8::PinScope<'_, '_>, bytes: &mut [u8]) -> bool {
    let Some(state) = scope.get_slot_mut::<DeterminismState>() else {
        return false;
    };
    if state.configuration.random_seed.is_none() {
        return false;
    }
    for chunk in bytes.chunks_mut(8) {
        let value = next_random_u64(&mut state.random_state).to_le_bytes();
        chunk.copy_from_slice(&value[..chunk.len()]);
    }
    true
}

fn random_f64(scope: &mut v8::PinScope<'_, '_>) -> Option<f64> {
    let state = scope.get_slot_mut::<DeterminismState>()?;
    state.configuration.random_seed?;
    let bits = next_random_u64(&mut state.random_state) >> 11;
    Some(bits as f64 * (1.0 / ((1_u64 << 53) as f64)))
}

fn next_random_u64(state: &mut u64) -> u64 {
    let mut value = *state;
    if value == 0 {
        value = 0x6a09_e667_f3bc_c909;
    }
    value ^= value >> 12;
    value ^= value << 25;
    value ^= value >> 27;
    *state = value;
    value.wrapping_mul(0x2545_f491_4f6c_dd1d)
}

fn system_epoch_milliseconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .unwrap_or(0.0)
}

fn clock_is_deterministic(scope: &v8::PinScope<'_, '_>) -> bool {
    scope
        .get_slot::<DeterminismState>()
        .is_some_and(|state| state.configuration.clock_epoch_ms.is_some())
}

fn random_is_deterministic(scope: &v8::PinScope<'_, '_>) -> bool {
    scope
        .get_slot::<DeterminismState>()
        .is_some_and(|state| state.configuration.random_seed.is_some())
}

fn install_date(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let global_id = global.get_identity_hash().get();
    if scope
        .get_slot::<DeterminismState>()
        .is_some_and(|state| state.original_date_by_global.contains_key(&global_id))
    {
        return Ok(());
    }
    let date_key = crate::webidl::string(scope, "Date")?;
    let date = global
        .get(scope, date_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
        .ok_or_else(|| "Date constructor is unavailable".to_owned())?;
    let replacement = crate::webidl::create_function(
        scope,
        "Date",
        7,
        v8::ConstructorBehavior::Allow,
        date_constructor,
    )?;
    let prototype_key = crate::webidl::string(scope, "prototype")?;
    let prototype = date
        .get(scope, prototype_key.into())
        .ok_or_else(|| "Date prototype is unavailable".to_owned())?;
    if replacement.define_own_property(
        scope,
        prototype_key.into(),
        prototype,
        v8::PropertyAttribute::DONT_ENUM
            | v8::PropertyAttribute::DONT_DELETE
            | v8::PropertyAttribute::READ_ONLY,
    ) != Some(true)
    {
        return Err("cannot attach deterministic Date.prototype".to_owned());
    }
    copy_date_static(scope, date, replacement, "parse")?;
    copy_date_static(scope, date, replacement, "UTC")?;
    let now =
        crate::webidl::create_function(scope, "now", 0, v8::ConstructorBehavior::Throw, date_now)?;
    let key = crate::webidl::string(scope, "now")?;
    if replacement.define_own_property(
        scope,
        key.into(),
        now.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot install deterministic Date.now".to_owned());
    }
    let original = v8::Global::new(scope, date);
    scope
        .get_slot_mut::<DeterminismState>()
        .ok_or_else(|| "deterministic state was not prepared".to_owned())?
        .original_date_by_global
        .insert(global_id, original);
    if global.define_own_property(
        scope,
        date_key.into(),
        replacement.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot install deterministic Date constructor".to_owned());
    }
    Ok(())
}

fn copy_date_static(
    scope: &mut v8::PinScope<'_, '_>,
    source: v8::Local<'_, v8::Function>,
    target: v8::Local<'_, v8::Function>,
    name: &str,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let value = source
        .get(scope, key.into())
        .ok_or_else(|| format!("Date.{name} is unavailable"))?;
    if target.define_own_property(scope, key.into(), value, v8::PropertyAttribute::DONT_ENUM)
        == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot copy Date.{name}"))
    }
}

fn date_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        let Some(date) = v8::Date::new(scope, date_epoch_milliseconds(scope)) else {
            return;
        };
        if let Some(text) = date.to_string(scope) {
            result.set(text.into());
        }
        return;
    }
    if arguments.length() == 0 {
        let Some(date) = v8::Date::new(scope, date_epoch_milliseconds(scope)) else {
            return;
        };
        result.set(date.into());
        return;
    }
    let global_id = scope
        .get_current_context()
        .global(scope)
        .get_identity_hash()
        .get();
    let original = scope
        .get_slot::<DeterminismState>()
        .and_then(|state| state.original_date_by_global.get(&global_id))
        .cloned();
    let Some(original) = original else {
        return;
    };
    let original = v8::Local::new(scope, &original);
    let values = (0..arguments.length())
        .map(|index| arguments.get(index))
        .collect::<Vec<_>>();
    if let Some(date) = original.new_instance(scope, &values) {
        result.set(date.into());
    }
}

fn date_now(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Number::new(scope, date_epoch_milliseconds(scope)).into());
}

fn install_math_random(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let global = scope.get_current_context().global(scope);
    let math_key = crate::webidl::string(scope, "Math")?;
    let math = global
        .get(scope, math_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "Math object is unavailable".to_owned())?;
    let random = crate::webidl::create_function(
        scope,
        "random",
        0,
        v8::ConstructorBehavior::Throw,
        math_random,
    )?;
    let key = crate::webidl::string(scope, "random")?;
    if math.define_own_property(
        scope,
        key.into(),
        random.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot install deterministic Math.random".to_owned())
    }
}

fn math_random(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = random_f64(scope) {
        result.set(v8::Number::new(scope, value).into());
    }
}
