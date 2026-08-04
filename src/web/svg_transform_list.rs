use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgTransformListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<super::svg_transform::TransformValue>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgTransformListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGTransformList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgTransformListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGTransformList",
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
    crate::webidl::define_method(scope, prototype, "consolidate", 0, consolidate)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "createSVGTransformFromMatrix",
        1,
        create_from_matrix,
    )?;
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
        .get_slot_mut::<SvgTransformListStore>()
        .ok_or_else(|| "SVGTransformList state was not prepared".to_owned())?
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
        return Err("cannot create SVGTransformList".to_owned());
    }
    scope
        .get_slot_mut::<SvgTransformListStore>()
        .ok_or_else(|| "SVGTransformList state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGTransformList': Illegal constructor",
    );
}

fn values(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<super::svg_transform::TransformValue>> {
    scope
        .get_slot::<SvgTransformListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn input_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<super::svg_transform::TransformValue> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    super::svg_transform::value(scope, object)
}

fn return_transform(
    scope: &mut v8::PinScope<'_, '_>,
    value: super::svg_transform::TransformValue,
    mut result: v8::ReturnValue<'_>,
) {
    match super::svg_transform::create(scope, value) {
        Ok(object) => result.set(object.into()),
        Err(error) => crate::webidl::throw_type_error(scope, &error),
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
    let Some(value) = input_value(scope, arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "appendItem requires an SVGTransform");
        return;
    };
    let Some(items) = scope
        .get_slot_mut::<SvgTransformListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    items.push(value);
    return_transform(scope, value, result);
}

fn clear(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(items) = scope
        .get_slot_mut::<SvgTransformListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        items.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn consolidate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(items) = values(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if items.is_empty() {
        result.set(v8::null(scope).into());
        return;
    }
    let mut matrix = super::svg_matrix::MatrixValue::identity();
    for item in &items {
        matrix = super::svg_matrix::product(matrix, item.matrix);
    }
    let value = super::svg_transform::TransformValue {
        kind: 1,
        matrix,
        angle: 0.0,
    };
    if let Some(current) = scope
        .get_slot_mut::<SvgTransformListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        current.clear();
        current.push(value);
    }
    return_transform(scope, value, result);
}

fn create_from_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if values(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(matrix) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "An SVGMatrix is required");
        return;
    };
    let Some(matrix) = super::svg_matrix::value(scope, matrix) else {
        crate::webidl::throw_type_error(scope, "An SVGMatrix is required");
        return;
    };
    return_transform(
        scope,
        super::svg_transform::TransformValue {
            kind: 1,
            matrix,
            angle: 0.0,
        },
        result,
    );
}

fn get_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(items) = values(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(value) = items.get(index).copied() else {
        crate::webidl::throw_type_error(scope, "SVGTransformList index is out of bounds");
        return;
    };
    return_transform(scope, value, result);
}

fn initialize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = input_value(scope, arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "initialize requires an SVGTransform");
        return;
    };
    let Some(items) = scope
        .get_slot_mut::<SvgTransformListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    items.clear();
    items.push(value);
    return_transform(scope, value, result);
}

fn insert_item_before(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = input_value(scope, arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "insertItemBefore requires an SVGTransform");
        return;
    };
    let requested = arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(items) = scope
        .get_slot_mut::<SvgTransformListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = requested.min(items.len());
    items.insert(index, value);
    return_transform(scope, value, result);
}

fn remove_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(items) = scope
        .get_slot_mut::<SvgTransformListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if index >= items.len() {
        crate::webidl::throw_type_error(scope, "SVGTransformList index is out of bounds");
        return;
    }
    let value = items.remove(index);
    return_transform(scope, value, result);
}

fn replace_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(value) = input_value(scope, arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "replaceItem requires an SVGTransform");
        return;
    };
    let index = arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(items) = scope
        .get_slot_mut::<SvgTransformListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if index >= items.len() {
        crate::webidl::throw_type_error(scope, "SVGTransformList index is out of bounds");
        return;
    }
    items[index] = value;
    return_transform(scope, value, result);
}
