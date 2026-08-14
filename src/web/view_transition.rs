use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ViewTransitionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ViewTransitionRecord>,
}

#[derive(Clone)]
struct ViewTransitionRecord {
    finished: v8::Global<v8::Promise>,
    ready: v8::Global<v8::Promise>,
    update_callback_done: v8::Global<v8::Promise>,
    types: v8::Global<v8::Object>,
    transition_root: Option<v8::Global<v8::Value>>,
    waits: Vec<v8::Global<v8::Promise>>,
    skipped: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ViewTransitionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ViewTransition", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ViewTransitionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ViewTransition",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "finished", get_finished)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ready", get_ready)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "updateCallbackDone",
        get_update_callback_done,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "types", get_types)?;
    crate::webidl::define_method(scope, prototype, "skipTransition", 0, skip_transition)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "transitionRoot",
        get_transition_root,
    )?;
    crate::webidl::define_method(scope, prototype, "waitUntil", 1, wait_until)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ViewTransitionStore>()
        .ok_or_else(|| "ViewTransition state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn resolved_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Promise>, String> {
    let resolver = v8::PromiseResolver::new(scope)
        .ok_or_else(|| "cannot create ViewTransition promise".to_owned())?;
    let promise = resolver.get_promise(scope);
    let undefined = v8::undefined(scope);
    let _ = resolver.resolve(scope, undefined.into());
    Ok(promise)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial_types: Vec<String>,
    transition_root: Option<v8::Local<'_, v8::Value>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ViewTransition".to_owned());
    }
    let finished = resolved_promise(scope)?;
    let ready = resolved_promise(scope)?;
    let update_callback_done = resolved_promise(scope)?;
    let types = super::view_transition_type_set::create(scope, initial_types)?;
    let transition_root = transition_root.map(|value| v8::Global::new(scope, value));
    let finished = v8::Global::new(scope, finished);
    let ready = v8::Global::new(scope, ready);
    let update_callback_done = v8::Global::new(scope, update_callback_done);
    let types = v8::Global::new(scope, types);
    scope
        .get_slot_mut::<ViewTransitionStore>()
        .ok_or_else(|| "ViewTransition state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ViewTransitionRecord {
                finished,
                ready,
                update_callback_done,
                types,
                transition_root,
                waits: Vec::new(),
                skipped: false,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ViewTransitionRecord> {
    scope
        .get_slot::<ViewTransitionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut ViewTransitionRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<ViewTransitionStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_promise(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    property_name: &str,
    select: impl FnOnce(&ViewTransitionRecord) -> v8::Global<v8::Promise>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &select(&record)).into());
    } else {
        let message = format!(
            "Failed to read the '{property_name}' property from 'ViewTransition': Illegal invocation"
        );
        if let Some(promise) = crate::webidl::rejected_type_error_promise(scope, &message) {
            result.set(promise.into());
        }
    }
}

fn get_finished(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_promise(s, a, r, "finished", |record| record.finished.clone())
}
fn get_ready(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_promise(s, a, r, "ready", |record| record.ready.clone())
}
fn get_update_callback_done(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_promise(s, a, r, "updateCallbackDone", |record| {
        record.update_callback_done.clone()
    })
}

fn get_types(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.types).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn skip_transition(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| record.skipped = true);
}

fn get_transition_root(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(root) = record.transition_root {
        result.set(v8::Local::new(scope, &root));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn wait_until(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(promise) = v8::Local::<v8::Promise>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "waitUntil requires a Promise");
        return;
    };
    let promise = v8::Global::new(scope, promise);
    update(scope, arguments.this(), |record| record.waits.push(promise));
}
