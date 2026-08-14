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
    let object = new_exotic_array(scope)?;
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

fn new_exotic_array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let template = v8::ObjectTemplate::new(scope);
    template.set_indexed_property_handler(
        v8::IndexedPropertyHandlerConfiguration::new()
            .getter(indexed_getter)
            .setter(indexed_setter)
            .query(indexed_query)
            .deleter(indexed_deleter)
            .enumerator(indexed_enumerator),
    );
    template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(named_getter)
            .setter(named_setter)
            .query(named_query)
            .deleter(named_deleter)
            .enumerator(named_enumerator),
    );
    template
        .new_instance(scope)
        .ok_or_else(|| "cannot create MimeTypeArray exotic object".to_owned())
}

fn has_index(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, index: u32) -> bool {
    items(scope, object).is_some_and(|items| (index as usize) < items.len())
}

fn property_name(scope: &mut v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    if key.is_symbol() {
        return None;
    }
    key.to_string(scope)
        .map(|key| key.to_rust_string_lossy(scope))
}

fn has_name(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> bool {
    items(scope, object).is_some_and(|items| {
        items
            .iter()
            .any(|mime| super::mime_type::mime_type(scope, *mime).as_deref() == Some(name))
    })
}

fn indexed_getter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "get", index, None);
    let Some(mime) =
        items(scope, arguments.holder()).and_then(|items| items.get(index as usize).copied())
    else {
        return v8::Intercepted::kNo;
    };
    result.set(mime.into());
    v8::Intercepted::kYes
}

fn indexed_query(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "has", index, None);
    if has_index(scope, arguments.holder(), index) {
        result.set_int32(1);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn indexed_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let length = items(scope, arguments.holder()).map_or(0, |items| items.len());
    let indices = (0..length)
        .map(|index| v8::Integer::new_from_unsigned(scope, index as u32).into())
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &indices));
}

fn named_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    items(scope, object)?
        .into_iter()
        .find(|mime| super::mime_type::mime_type(scope, *mime).as_deref() == Some(name))
}

fn named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "get", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    let Some(mime) = named_value(scope, arguments.holder(), &name) else {
        return v8::Intercepted::kNo;
    };
    result.set(mime.into());
    v8::Intercepted::kYes
}

fn named_query(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "has", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if has_name(scope, arguments.holder(), &name) {
        result.set_int32(3);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let names = items(scope, arguments.holder())
        .unwrap_or_default()
        .iter()
        .filter_map(|mime| {
            super::mime_type::mime_type(scope, *mime)
                .and_then(|name| v8::String::new(scope, &name))
                .map(Into::into)
        })
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &names));
}

fn indexed_setter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "set", index, Some(value));
    if !has_index(scope, arguments.holder(), index) {
        return v8::Intercepted::kNo;
    }
    if arguments.should_throw_on_error() {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to set an indexed property [{index}] on 'MimeTypeArray': Indexed property setter is not supported."
            ),
        );
    } else {
        result.set_bool(false);
    }
    v8::Intercepted::kYes
}

fn indexed_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "delete", index, None);
    if !has_index(scope, arguments.holder(), index) {
        return v8::Intercepted::kNo;
    }
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn named_setter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "set", key, Some(value));
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if !has_name(scope, arguments.holder(), &name) {
        return v8::Intercepted::kNo;
    }
    if arguments.should_throw_on_error() {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to set a named property '{name}' on 'MimeTypeArray': Named property setter is not supported."
            ),
        );
    } else {
        result.set_bool(false);
    }
    v8::Intercepted::kYes
}

fn named_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "delete", key, None);
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if !has_name(scope, arguments.holder(), &name) {
        return v8::Intercepted::kNo;
    }
    result.set_bool(false);
    v8::Intercepted::kYes
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
    result: v8::ReturnValue<'_>,
) {
    crate::webidl::return_array_like_iterator(
        scope,
        arguments.this(),
        crate::webidl::ArrayLikeIteratorKind::Values,
        result,
    );
}
