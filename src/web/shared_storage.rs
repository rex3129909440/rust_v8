use std::collections::HashMap;

#[derive(Clone)]
struct SharedStorageRecord {
    entries: HashMap<String, String>,
    worklet: v8::Global<v8::Object>,
    next_selection: u64,
}

#[derive(Default)]
pub(crate) struct SharedStorageStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SharedStorageRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SharedStorageStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SharedStorage", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SharedStorageStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SharedStorage",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "append", 2, append)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "delete", 1, delete)?;
    crate::webidl::define_method(scope, prototype, "set", 2, set)?;
    if crate::browser_surface::current_version(scope).major() <= 147 {
        super::shared_storage_get::define(scope, prototype)?;
    }
    crate::webidl::define_method(scope, prototype, "batchUpdate", 1, batch_update)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "worklet", get_worklet)?;
    crate::webidl::define_method(scope, prototype, "createWorklet", 1, create_worklet)?;
    crate::webidl::define_method(scope, prototype, "run", 1, run)?;
    crate::webidl::define_method(scope, prototype, "selectURL", 2, select_url)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SharedStorageStore>()
        .ok_or_else(|| "SharedStorage state was not prepared".to_owned())?
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
        return Err("cannot create SharedStorage".to_owned());
    }
    let worklet = super::shared_storage_worklet::create(scope)?;
    let record = SharedStorageRecord {
        entries: HashMap::new(),
        worklet: v8::Global::new(scope, worklet),
        next_selection: 0,
    };
    scope
        .get_slot_mut::<SharedStorageStore>()
        .ok_or_else(|| "SharedStorage state is unavailable".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SharedStorage': Illegal constructor",
    )
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SharedStorageRecord> {
    scope
        .get_slot::<SharedStorageStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn has_record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut SharedStorageRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<SharedStorageStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(record);
    true
}

fn resolve_undefined(scope: &mut v8::PinScope<'_, '_>, mut result: v8::ReturnValue<'_>) {
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

fn require_arguments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    method: &str,
    count: i32,
) -> bool {
    if arguments.length() >= count {
        true
    } else {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to execute '{method}' on 'SharedStorage': {count} arguments required, but only {} present.",
                arguments.length()
            ),
        );
        false
    }
}

fn append(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(scope, "SharedStorage", "append", result);
        return;
    }
    if !require_arguments(scope, &arguments, "append", 2) {
        return;
    }
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    if update(scope, arguments.this(), |record| {
        record
            .entries
            .entry(key)
            .and_modify(|existing| existing.push_str(&value))
            .or_insert(value);
    }) {
        resolve_undefined(scope, result);
    }
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(scope, "SharedStorage", "clear", result);
        return;
    }
    if update(scope, arguments.this(), |record| record.entries.clear()) {
        resolve_undefined(scope, result);
    }
}

fn delete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(scope, "SharedStorage", "delete", result);
        return;
    }
    if !require_arguments(scope, &arguments, "delete", 1) {
        return;
    }
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    if update(scope, arguments.this(), |record| {
        record.entries.remove(&key);
    }) {
        resolve_undefined(scope, result);
    }
}

fn set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(scope, "SharedStorage", "set", result);
        return;
    }
    if !require_arguments(scope, &arguments, "set", 2) {
        return;
    }
    let key = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = crate::webidl::value_to_string(scope, arguments.get(1));
    let ignore_if_present = super::shared_storage_modifier_method::option_bool(
        scope,
        arguments.get(2),
        "ignoreIfPresent",
    );
    if update(scope, arguments.this(), |record| {
        if !ignore_if_present || !record.entries.contains_key(&key) {
            record.entries.insert(key, value);
        }
    }) {
        resolve_undefined(scope, result);
    }
}

fn apply_operation(
    record: &mut SharedStorageRecord,
    operation: super::shared_storage_modifier_method::SharedStorageOperation,
) {
    use super::shared_storage_modifier_method::SharedStorageOperation;
    match operation {
        SharedStorageOperation::Append {
            key,
            value,
            with_lock,
        } => {
            let _ = with_lock;
            record
                .entries
                .entry(key)
                .and_modify(|existing| existing.push_str(&value))
                .or_insert(value);
        }
        SharedStorageOperation::Clear { with_lock } => {
            let _ = with_lock;
            record.entries.clear();
        }
        SharedStorageOperation::Delete { key, with_lock } => {
            let _ = with_lock;
            record.entries.remove(&key);
        }
        SharedStorageOperation::Set {
            key,
            value,
            ignore_if_present,
            with_lock,
        } => {
            let _ = with_lock;
            if !ignore_if_present || !record.entries.contains_key(&key) {
                record.entries.insert(key, value);
            }
        }
    }
}

fn batch_update(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "SharedStorage",
            "batchUpdate",
            result,
        );
        return;
    }
    if !require_arguments(scope, &arguments, "batchUpdate", 1) {
        return;
    }
    let Ok(methods) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "SharedStorage batch methods must be an array");
        return;
    };
    let mut operations = Vec::new();
    for index in 0..methods.length() {
        let Some(value) = methods.get_index(scope, index) else {
            continue;
        };
        let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
            crate::webidl::throw_type_error(scope, "Invalid shared storage modifier method");
            return;
        };
        let Some(operation) = super::shared_storage_modifier_method::operation(scope, object)
        else {
            crate::webidl::throw_type_error(scope, "Invalid shared storage modifier method");
            return;
        };
        operations.push(operation);
    }
    if update(scope, arguments.this(), |record| {
        for operation in operations {
            apply_operation(record, operation);
        }
    }) {
        resolve_undefined(scope, result);
    }
}

fn get_worklet(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.worklet).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn create_worklet(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "SharedStorage",
            "createWorklet",
            result,
        );
        return;
    }
    if !require_arguments(scope, &arguments, "createWorklet", 1) {
        return;
    }
    let Ok(worklet) = super::shared_storage_worklet::create(scope) else {
        return;
    };
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, worklet.into()) {
        result.set(promise.into());
    }
}

fn run(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(scope, "SharedStorage", "run", result);
        return;
    }
    if !require_arguments(scope, &arguments, "run", 1) {
        return;
    }
    resolve_undefined(scope, result);
}

fn select_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "SharedStorage",
            "selectURL",
            result,
        );
        return;
    }
    if !require_arguments(scope, &arguments, "selectURL", 2) {
        return;
    }
    let mut next = 0;
    if !update(scope, arguments.this(), |record| {
        record.next_selection = record.next_selection.saturating_add(1);
        next = record.next_selection;
    }) {
        return;
    }
    if let Some(value) = v8::String::new(scope, &format!("urn:uuid:shared-storage-{next}")) {
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
            result.set(promise.into());
        }
    }
}
