use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgStringListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<String>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgStringListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGStringList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgStringListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGStringList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "numberOfItems", get_length)?;
    crate::webidl::define_method(scope, prototype, "appendItem", 1, append_item)?;
    crate::webidl::define_method(scope, prototype, "clear", 0, clear)?;
    crate::webidl::define_method(scope, prototype, "getItem", 1, get_item)?;
    crate::webidl::define_method(scope, prototype, "initialize", 1, initialize)?;
    crate::webidl::define_method(scope, prototype, "insertItemBefore", 2, insert_item_before)?;
    crate::webidl::define_method(scope, prototype, "removeItem", 1, remove_item)?;
    crate::webidl::define_method(scope, prototype, "replaceItem", 2, replace_item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgStringListStore>()
        .ok_or_else(|| "SVGStringList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: Vec<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGStringList".to_owned());
    }
    scope
        .get_slot_mut::<SvgStringListStore>()
        .ok_or_else(|| "SVGStringList state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), initial);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGStringList': Illegal constructor",
    );
}

fn values(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Vec<String>> {
    scope
        .get_slot::<SvgStringListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn text(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> String {
    crate::webidl::value_to_string(scope, value)
}

fn return_text(scope: &v8::PinScope<'_, '_>, value: &str, mut result: v8::ReturnValue<'_>) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = values(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, values.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn append_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = text(scope, arguments.get(0));
    let Some(values) = scope
        .get_slot_mut::<SvgStringListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    values.push(value.clone());
    return_text(scope, &value, result);
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(values) = scope
        .get_slot_mut::<SvgStringListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        values.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(values) = values(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(value) = values.get(index) else {
        crate::webidl::throw_type_error(scope, "SVGStringList index is out of bounds");
        return;
    };
    return_text(scope, value, result);
}

fn initialize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = text(scope, arguments.get(0));
    let Some(values) = scope
        .get_slot_mut::<SvgStringListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    values.clear();
    values.push(value.clone());
    return_text(scope, &value, result);
}

fn insert_item_before(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = text(scope, arguments.get(0));
    let requested = arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(values) = scope
        .get_slot_mut::<SvgStringListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = requested.min(values.len());
    values.insert(index, value.clone());
    return_text(scope, &value, result);
}

fn remove_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(values) = scope
        .get_slot_mut::<SvgStringListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if index >= values.len() {
        crate::webidl::throw_type_error(scope, "SVGStringList index is out of bounds");
        return;
    }
    let value = values.remove(index);
    return_text(scope, &value, result);
}

fn replace_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let value = text(scope, arguments.get(0));
    let index = arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(values) = scope
        .get_slot_mut::<SvgStringListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if index >= values.len() {
        crate::webidl::throw_type_error(scope, "SVGStringList index is out of bounds");
        return;
    }
    values[index] = value.clone();
    return_text(scope, &value, result);
}
