use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgRectStore {
    constructor: crate::webidl::RealmConstructor,
    next_group: u64,
    objects: HashMap<i32, u64>,
    values: HashMap<u64, RectValue>,
}

#[derive(Clone, Copy)]
pub(crate) struct RectValue {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgRectStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGRect", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgRectStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGRect",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "x", get_x, set_x)?;
    crate::webidl::define_accessor(scope, prototype, "y", get_y, set_y)?;
    crate::webidl::define_accessor(scope, prototype, "width", get_width, set_width)?;
    crate::webidl::define_accessor(scope, prototype, "height", get_height, set_height)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgRectStore>()
        .ok_or_else(|| "SVGRect state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create_pair<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: RectValue,
) -> Result<(v8::Local<'s, v8::Object>, v8::Local<'s, v8::Object>), String> {
    let group = {
        let store = scope
            .get_slot_mut::<SvgRectStore>()
            .ok_or_else(|| "SVGRect state was not prepared".to_owned())?;
        store.next_group += 1;
        let group = store.next_group;
        store.values.insert(group, value);
        group
    };
    let base = create_for_group(scope, group)?;
    let animated = create_for_group(scope, group)?;
    Ok((base, animated))
}

fn create_for_group<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    group: u64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGRect".to_owned());
    }
    scope
        .get_slot_mut::<SvgRectStore>()
        .ok_or_else(|| "SVGRect state was not prepared".to_owned())?
        .objects
        .insert(object.get_identity_hash().get(), group);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'SVGRect': Illegal constructor");
}

fn value(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<RectValue> {
    let store = scope.get_slot::<SvgRectStore>()?;
    let group = store.objects.get(&object.get_identity_hash().get())?;
    store.values.get(group).copied()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut RectValue),
) {
    let Some(store) = scope.get_slot_mut::<SvgRectStore>() else {
        return;
    };
    let Some(group) = store
        .objects
        .get(&object.get_identity_hash().get())
        .copied()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = store.values.get_mut(&group) {
        change(value);
    }
}

fn return_number(scope: &v8::PinScope<'_, '_>, value: f64, mut result: v8::ReturnValue<'_>) {
    result.set(v8::Number::new(scope, value).into());
}

fn get_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        return_number(scope, value.x, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |current| current.x = value);
}

fn get_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        return_number(scope, value.y, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |current| current.y = value);
}

fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        return_number(scope, value.width, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |current| current.width = value);
}

fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        return_number(scope, value.height, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |current| current.height = value);
}
