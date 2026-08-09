use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct BarPropStore {
    constructor: crate::webidl::RealmConstructor,
    values: HashMap<i32, bool>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BarPropStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "BarProp", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<BarPropStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BarProp",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "visible", get_visible)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BarPropStore>()
        .ok_or_else(|| "BarProp state was not prepared".to_owned())?
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

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    visible: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create BarProp".to_owned());
    }
    scope
        .get_slot_mut::<BarPropStore>()
        .ok_or_else(|| "BarProp state was not prepared".to_owned())?
        .values
        .insert(object.get_identity_hash().get(), visible);
    Ok(object)
}

pub(crate) fn visible_for_current_page(scope: &v8::PinScope<'_, '_>) -> bool {
    !crate::fingerprint::edge(scope)
        .document
        .is_popup
        .unwrap_or(false)
}

fn get_visible(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope.get_slot::<BarPropStore>().and_then(|store| {
        store
            .values
            .get(&arguments.this().get_identity_hash().get())
    }) {
        result.set(v8::Boolean::new(scope, *value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
