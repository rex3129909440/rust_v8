use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssTransformComponentStore {
    constructor: crate::webidl::RealmConstructor,
    is_2d: HashMap<i32, bool>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssTransformComponentStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSTransformComponent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CssTransformComponentStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSTransformComponent",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "is2D", get_is_2d, set_is_2d_callback)?;
    crate::webidl::define_method(scope, prototype, "toMatrix", 0, to_matrix)?;
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssTransformComponentStore>()
        .ok_or_else(|| "CSSTransformComponent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    is_2d: bool,
) {
    scope
        .get_slot_mut::<CssTransformComponentStore>()
        .expect("CSSTransformComponent state")
        .is_2d
        .insert(object.get_identity_hash().get(), is_2d);
}

pub(crate) fn is_component(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<CssTransformComponentStore>()
        .is_some_and(|store| store.is_2d.contains_key(&object.get_identity_hash().get()))
}

pub(crate) fn is_2d(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<bool> {
    scope
        .get_slot::<CssTransformComponentStore>()?
        .is_2d
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn set_is_2d(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: bool,
) {
    if let Some(current) = scope
        .get_slot_mut::<CssTransformComponentStore>()
        .and_then(|store| store.is_2d.get_mut(&object.get_identity_hash().get()))
    {
        *current = value;
    }
}

fn get_is_2d(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = is_2d(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_is_2d_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !is_component(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).boolean_value(scope);
    set_is_2d(scope, arguments.this(), value);
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    super::css_skew_x::serialize(scope, object)
        .or_else(|| super::css_skew_y::serialize(scope, object))
        .or_else(|| super::css_skew::serialize(scope, object))
        .or_else(|| super::css_scale::serialize(scope, object))
        .or_else(|| super::css_rotate::serialize(scope, object))
        .or_else(|| super::css_perspective::serialize(scope, object))
        .or_else(|| super::css_matrix_component::serialize(scope, object))
        .or_else(|| super::css_translate::serialize_component(scope, object))
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    super::css_skew_x::matrix(scope, object)
        .or_else(|| super::css_skew_y::matrix(scope, object))
        .or_else(|| super::css_skew::matrix(scope, object))
        .or_else(|| super::css_scale::matrix(scope, object))
        .or_else(|| super::css_rotate::matrix(scope, object))
        .or_else(|| super::css_perspective::matrix(scope, object))
        .or_else(|| super::css_matrix_component::matrix(scope, object))
        .or_else(|| super::css_translate::matrix(scope, object))
}

fn to_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(matrix) = matrix(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match super::dom_matrix::create_from_matrix(scope, matrix) {
        Ok(matrix) => result.set(matrix.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = serialize(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CssTransformComponentStore>() {
        store.constructor.remove(realm_id);
    }
}
