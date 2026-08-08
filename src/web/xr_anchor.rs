use std::collections::HashMap;

#[derive(Clone)]
struct AnchorRecord {
    space: v8::Global<v8::Object>,
    deleted: bool,
}

#[derive(Default)]
pub(crate) struct XrAnchorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AnchorRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrAnchorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRAnchor", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrAnchorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRAnchor",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "anchorSpace", get_anchor_space)?;
    crate::webidl::define_method(scope, prototype, "delete", 0, delete_anchor)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrAnchorStore>()
        .ok_or_else(|| "XRAnchor state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRAnchor".to_owned());
    }
    let space = super::xr_space::create(scope)?;
    let space = v8::Global::new(scope, space);
    let identity = object.get_identity_hash().get();
    scope
        .get_slot_mut::<XrAnchorStore>()
        .ok_or_else(|| "XRAnchor state missing".to_owned())?
        .records
        .insert(
            identity,
            AnchorRecord {
                space,
                deleted: false,
            },
        );
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<AnchorRecord> {
    scope
        .get_slot::<XrAnchorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_anchor_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(anchor) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &anchor.space).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn delete_anchor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(anchor) = scope.get_slot_mut::<XrAnchorStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    anchor.deleted = true;
    result.set(v8::undefined(scope).into());
}
