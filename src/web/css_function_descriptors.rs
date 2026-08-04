use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct CssFunctionDescriptorsStore {
    constructor: crate::webidl::RealmConstructor,
    native_objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssFunctionDescriptorsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSFunctionDescriptors", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssFunctionDescriptorsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSFunctionDescriptors",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "result", get_result, set_result)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_style_declaration::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssFunctionDescriptorsStore>()
        .ok_or_else(|| "CSSFunctionDescriptors state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    declarations: &str,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let descriptors = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, descriptors, prototype.into()) != Some(true) {
        return Err("cannot create CSSFunctionDescriptors".to_owned());
    }
    super::css_style_declaration::attach(scope, descriptors, declarations, parent_rule, None)?;
    scope
        .get_slot_mut::<CssFunctionDescriptorsStore>()
        .ok_or_else(|| "CSSFunctionDescriptors state was not prepared".to_owned())?
        .native_objects
        .insert(descriptors.get_identity_hash().get());
    Ok(descriptors)
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<CssFunctionDescriptorsStore>()
        .is_some_and(|store| {
            store
                .native_objects
                .contains(&object.get_identity_hash().get())
        })
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CSSFunctionDescriptors': Illegal constructor",
    );
}

fn get_result(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::css_style_declaration::property_value(scope, arguments.this(), "result")
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_result(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let _ =
        super::css_style_declaration::set_property_value(scope, arguments.this(), "result", value);
}
