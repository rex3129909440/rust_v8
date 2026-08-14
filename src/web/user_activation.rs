use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct UserActivationStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashMap<i32, ActivationObject>,
    windows: HashMap<i32, WindowActivationState>,
}

#[derive(Clone)]
struct ActivationObject {
    realm_id: i32,
    window_id: i32,
}

#[derive(Clone)]
struct WindowActivationState {
    has_been_active: bool,
    transient_until_ms: Option<f64>,
}

const TRANSIENT_ACTIVATION_DURATION_MS: f64 = 5_000.0;

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(UserActivationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "UserActivation", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<UserActivationStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "UserActivation",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "hasBeenActive",
        get_has_been_active,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "isActive", get_is_active)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<UserActivationStore>()
        .ok_or_else(|| "UserActivation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    has_been_active: bool,
    is_active: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create UserActivation".to_owned());
    }
    let realm_id = crate::webidl::realm_id(scope);
    let window_id = scope
        .get_current_context()
        .global(scope)
        .get_identity_hash()
        .get();
    let transient_until_ms = is_active.then(|| {
        crate::determinism::monotonic_snapshot_milliseconds(scope)
            + TRANSIENT_ACTIVATION_DURATION_MS
    });
    let store = scope
        .get_slot_mut::<UserActivationStore>()
        .ok_or_else(|| "UserActivation state was not prepared".to_owned())?;
    store.objects.insert(
        object.get_identity_hash().get(),
        ActivationObject {
            realm_id,
            window_id,
        },
    );
    store
        .windows
        .entry(window_id)
        .or_insert(WindowActivationState {
            has_been_active,
            transient_until_ms,
        });
    Ok(object)
}

fn state_for_object(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<WindowActivationState> {
    let store = scope.get_slot::<UserActivationStore>()?;
    let object = store.objects.get(&object.get_identity_hash().get())?;
    store.windows.get(&object.window_id).cloned()
}

pub(crate) fn current_realm_is_active(scope: &v8::PinScope<'_, '_>) -> bool {
    let window_id = scope
        .get_current_context()
        .global(scope)
        .get_identity_hash()
        .get();
    let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
    scope
        .get_slot::<UserActivationStore>()
        .is_some_and(|store| {
            store
                .windows
                .get(&window_id)
                .and_then(|state| state.transient_until_ms)
                .is_some_and(|until| now < until)
        })
}

pub(crate) fn current_realm_has_been_active(scope: &v8::PinScope<'_, '_>) -> bool {
    let window_id = scope
        .get_current_context()
        .global(scope)
        .get_identity_hash()
        .get();
    scope
        .get_slot::<UserActivationStore>()
        .is_some_and(|store| {
            store
                .windows
                .get(&window_id)
                .is_some_and(|state| state.has_been_active)
        })
}

pub(crate) fn activate_current_realm(scope: &mut v8::PinScope<'_, '_>) {
    let affected_windows =
        super::html_i_frame_element::user_activation_notification_window_ids(scope);
    let until = crate::determinism::monotonic_snapshot_milliseconds(scope)
        + TRANSIENT_ACTIVATION_DURATION_MS;
    if let Some(store) = scope.get_slot_mut::<UserActivationStore>() {
        for window_id in affected_windows {
            if let Some(state) = store.windows.get_mut(&window_id) {
                state.has_been_active = true;
                state.transient_until_ms = Some(until);
            }
        }
    }
}

pub(crate) fn consume_current_realm(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let active = current_realm_is_active(scope);
    if !active {
        return false;
    }
    let affected_windows =
        super::html_i_frame_element::user_activation_consumption_window_ids(scope);
    if let Some(store) = scope.get_slot_mut::<UserActivationStore>() {
        for window_id in affected_windows {
            if let Some(state) = store.windows.get_mut(&window_id) {
                state.transient_until_ms = None;
            }
        }
    }
    true
}

fn get_has_been_active(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = state_for_object(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, state.has_been_active).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_is_active(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = state_for_object(scope, arguments.this()) {
        let now = crate::determinism::monotonic_snapshot_milliseconds(scope);
        result.set(
            v8::Boolean::new(
                scope,
                state.transient_until_ms.is_some_and(|until| now < until),
            )
            .into(),
        );
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UserActivationStore>() {
        store.constructor.remove(realm_id);
        store
            .objects
            .retain(|_, object| object.realm_id != realm_id);
        let live_windows = store
            .objects
            .values()
            .map(|object| object.window_id)
            .collect::<std::collections::HashSet<_>>();
        store
            .windows
            .retain(|window_id, _| live_windows.contains(window_id));
    }
}
