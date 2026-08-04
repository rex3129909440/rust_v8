use std::collections::HashMap;

#[derive(Clone, Default)]
struct ClipboardRecord {
    items: Vec<v8::Global<v8::Object>>,
    text: String,
    onchange: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct ClipboardStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ClipboardRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ClipboardStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Clipboard", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ClipboardStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::event_target::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Clipboard",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onclipboardchange",
        get_onchange,
        set_onchange,
    )?;
    crate::webidl::define_method(scope, prototype, "read", 0, read)?;
    crate::webidl::define_method(scope, prototype, "readText", 0, read_text)?;
    crate::webidl::define_method(scope, prototype, "write", 1, write)?;
    crate::webidl::define_method(scope, prototype, "writeText", 1, write_text)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ClipboardStore>()
        .ok_or_else(|| "Clipboard state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'Clipboard': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Clipboard".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<ClipboardStore>()
        .ok_or_else(|| "Clipboard state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), ClipboardRecord::default());
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ClipboardRecord> {
    scope
        .get_slot::<ClipboardStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}

fn read(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.items.len() as i32);
    for (index, item) in record.items.iter().enumerate() {
        let item = v8::Local::new(scope, item);
        let _ = array.set_index(scope, index as u32, item.into());
    }
    resolve(scope, array.into(), result);
}

fn read_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(text) = v8::String::new(scope, &record.text) {
        resolve(scope, text.into(), result);
    }
}

fn write(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Ok(items) = v8::Local::<v8::Array>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Clipboard.write requires a sequence");
        return;
    };
    let mut stored = Vec::new();
    for index in 0..items.length() {
        if let Some(item) = items
            .get_index(scope, index)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        {
            stored.push(v8::Global::new(scope, item));
        }
    }
    let Some(record) = scope.get_slot_mut::<ClipboardStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.items = stored;
    resolve(scope, v8::undefined(scope).into(), result);
}

fn write_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = scope.get_slot_mut::<ClipboardStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.text = text;
    resolve(scope, v8::undefined(scope).into(), result);
}

fn get_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = record(scope, arguments.this()).and_then(|record| record.onchange);
    super::window_event_handler_support::return_handler(scope, handler, result);
}

fn set_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<ClipboardStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.onchange = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
