use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceMarkStore {
    constructor: crate::webidl::RealmConstructor,
    details: HashMap<i32, v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceMarkStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceMark", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<PerformanceMarkStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceMark",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "detail", get_detail)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceMarkStore>()
        .ok_or_else(|| "PerformanceMark state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PerformanceMark': 1 argument required",
        );
        return;
    }
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let start_time = options
        .map(|options| super::event::number_property(scope, options, "startTime", 0.0))
        .unwrap_or(0.0);
    if start_time < 0.0 {
        crate::webidl::throw_type_error(
            scope,
            "A PerformanceMark cannot have a negative start time",
        );
        return;
    }
    let detail = options
        .and_then(|options| property(scope, options, "detail"))
        .map(|value| clone_value(scope, value))
        .unwrap_or_else(|| v8::null(scope).into());
    attach(scope, arguments.this(), name, start_time, detail);
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    start_time: f64,
    detail: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if start_time < 0.0 {
        return Err("A PerformanceMark cannot have a negative start time".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let mark = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, mark, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceMark".to_owned());
    }
    let detail = clone_value(scope, detail);
    attach(scope, mark, name, start_time, detail);
    Ok(mark)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: String,
    start_time: f64,
    detail: v8::Local<'_, v8::Value>,
) {
    super::performance_entry::attach(scope, object, name, "mark".to_owned(), start_time, 0.0);
    let detail = v8::Global::new(scope, detail);
    scope
        .get_slot_mut::<PerformanceMarkStore>()
        .expect("PerformanceMark state")
        .details
        .insert(object.get_identity_hash().get(), detail);
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn clone_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
) -> v8::Local<'s, v8::Value> {
    let Ok(source) = v8::Local::<v8::Object>::try_from(value) else {
        return value;
    };
    if source.is_array() {
        let source_array = v8::Local::<v8::Array>::try_from(source).ok();
        let length = source_array.map(|array| array.length()).unwrap_or(0);
        let target = v8::Array::new(scope, length as i32);
        for index in 0..length {
            if let Some(item) = source.get_index(scope, index) {
                let _ = target.set_index(scope, index, clone_value(scope, item));
            }
        }
        return target.into();
    }
    let target = v8::Object::new(scope);
    if let Some(names) = source.get_own_property_names(scope, Default::default()) {
        for index in 0..names.length() {
            let Some(name_value) = names.get_index(scope, index) else {
                continue;
            };
            let Ok(name) = v8::Local::<v8::Name>::try_from(name_value) else {
                continue;
            };
            let Some(item) = source.get(scope, name.into()) else {
                continue;
            };
            let _ = target.create_data_property(scope, name, clone_value(scope, item));
        }
    }
    target.into()
}

fn get_detail(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(detail) = scope.get_slot::<PerformanceMarkStore>().and_then(|store| {
        store
            .details
            .get(&arguments.this().get_identity_hash().get())
    }) {
        result.set(v8::Local::new(scope, detail));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PerformanceMarkStore>() {
        store.constructor.remove(realm_id);
    }
}
