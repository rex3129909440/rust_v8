use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgFeMergeNodeElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) inputs: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgFeMergeNodeElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGFEMergeNodeElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgFeMergeNodeElementStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGFEMergeNodeElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_fe_merge_node_element_in1_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgFeMergeNodeElementStore>()
        .ok_or_else(|| "SVGFEMergeNodeElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object =
        super::svg_element::create_with_constructor(scope, constructor, "feMergeNode", owner)?;
    let input = super::svg_animated_string::create(scope, "")?;
    let input = v8::Global::new(scope, input);
    scope
        .get_slot_mut::<SvgFeMergeNodeElementStore>()
        .ok_or_else(|| "SVGFEMergeNodeElement state was not prepared".to_owned())?
        .inputs
        .insert(object.get_identity_hash().get(), input);
    Ok(object)
}
pub(crate) fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'SVGFEMergeNodeElement': Illegal constructor",
    )
}
pub(crate) fn get_input(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let input = s
        .get_slot::<SvgFeMergeNodeElementStore>()
        .and_then(|store| store.inputs.get(&a.this().get_identity_hash().get()))
        .cloned();
    if let Some(input) = input {
        r.set(v8::Local::new(s, &input).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
