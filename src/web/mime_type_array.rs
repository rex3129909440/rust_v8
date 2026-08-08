use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MimeTypeArrayStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MimeTypeArrayStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MimeTypeArray", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MimeTypeArrayStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MimeTypeArray",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::define_method(scope, prototype, "namedItem", 1, named_item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let iterator =
        crate::webidl::create_function(scope, "values", 0, v8::ConstructorBehavior::Throw, values)?;
    if prototype.define_own_property(
        scope,
        v8::Symbol::get_iterator(scope).into(),
        iterator.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define MimeTypeArray iterator".to_owned());
    }
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MimeTypeArrayStore>()
        .ok_or_else(|| "MimeTypeArray state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    plugins: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MimeTypeArray".to_owned());
    }
    let plugin_items = super::plugin_array::items(scope, plugins).unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    let mut values = Vec::new();
    for plugin in plugin_items {
        let plugin = v8::Local::new(scope, &plugin);
        for mime in super::plugin::mime_types(scope, plugin).unwrap_or_default() {
            let Some(name) = super::mime_type::mime_type(scope, mime) else {
                continue;
            };
            if !seen.insert(name.clone()) {
                continue;
            }
            define_indexed(scope, object, &values.len().to_string(), mime.into());
            define_indexed(scope, object, &name, mime.into());
            values.push(v8::Global::new(scope, mime));
        }
    }
    scope
        .get_slot_mut::<MimeTypeArrayStore>()
        .ok_or_else(|| "MimeTypeArray state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), values);
    Ok(object)
}

fn define_indexed(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::READ_ONLY);
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'MimeTypeArray': Illegal constructor",
    );
}

fn items<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    scope
        .get_slot::<MimeTypeArrayStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .map(|items| {
            items
                .iter()
                .map(|item| v8::Local::new(scope, item))
                .collect()
        })
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(items) = items(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Integer::new_from_unsigned(scope, items.len() as u32).into());
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(items) = items(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(item) = items.get(index) {
        result.set((*item).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(items) = items(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let found = items
        .iter()
        .find(|mime| super::mime_type::mime_type(scope, **mime).is_some_and(|value| value == name));
    if let Some(mime) = found {
        result.set((*mime).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(items) = items(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, items.len() as i32);
    for (index, item) in items.iter().enumerate() {
        let _ = array.set_index(scope, index as u32, (*item).into());
    }
    let Some(key) = v8::String::new(scope, "values") else {
        return;
    };
    let Some(function) = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())
    else {
        return;
    };
    if let Some(iterator) = function.call(scope, array.into(), &[]) {
        result.set(iterator);
    }
}
