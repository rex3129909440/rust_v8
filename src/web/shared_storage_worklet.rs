use std::collections::HashMap;

#[derive(Clone, Default)]
struct SharedStorageWorkletRecord {
    modules: Vec<String>,
    runs: u64,
}

#[derive(Default)]
pub(crate) struct SharedStorageWorkletStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SharedStorageWorkletRecord>,
    next_selection: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SharedStorageWorkletStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SharedStorageWorklet", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SharedStorageWorkletStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SharedStorageWorklet",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "addModule", 1, add_module)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, prototype, "run", 1, run)?;
    crate::webidl::define_method(scope, prototype, "selectURL", 2, select_url)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SharedStorageWorkletStore>()
        .ok_or_else(|| "SharedStorageWorklet state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SharedStorageWorklet".to_owned());
    }
    scope
        .get_slot_mut::<SharedStorageWorkletStore>()
        .ok_or_else(|| "SharedStorageWorklet state is unavailable".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            SharedStorageWorkletRecord::default(),
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SharedStorageWorklet': Illegal constructor",
    )
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<SharedStorageWorkletStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

fn resolve_undefined(scope: &mut v8::PinScope<'_, '_>, mut result: v8::ReturnValue<'_>) {
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn add_module(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'addModule' on 'Worklet': 1 argument required, but only 0 present.",
        );
        return;
    }
    let url = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<SharedStorageWorkletStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !record.modules.contains(&url) {
        record.modules.push(url);
    }
    resolve_undefined(scope, result);
}

fn run(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'run' on 'SharedStorageWorklet': 1 argument required, but only 0 present.",
        );
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<SharedStorageWorkletStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.runs = record.runs.saturating_add(1);
    }
    resolve_undefined(scope, result);
}

fn select_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute 'selectURL' on 'SharedStorageWorklet': 2 arguments required, but only {} present.",
                arguments.length()
            ),
        );
        return;
    }
    let next = {
        let store = scope
            .get_slot_mut::<SharedStorageWorkletStore>()
            .expect("SharedStorageWorklet state");
        store.next_selection = store.next_selection.saturating_add(1);
        store.next_selection
    };
    let value = v8::String::new(scope, &format!("urn:uuid:shared-storage-{next}"));
    if let Some(value) = value {
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
            result.set(promise.into());
        }
    }
}
