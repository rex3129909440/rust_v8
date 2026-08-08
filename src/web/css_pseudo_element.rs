use std::collections::HashMap;

#[derive(Clone)]
struct CssPseudoElementRecord {
    pseudo_type: String,
    element: v8::Global<v8::Object>,
    parent: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssPseudoElementStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssPseudoElementRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssPseudoElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSPseudoElement", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssPseudoElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSPseudoElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "element", get_element)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "parent", get_parent)?;
    crate::webidl::define_method(scope, prototype, "pseudo", 1, pseudo)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssPseudoElementStore>()
        .ok_or_else(|| "CSSPseudoElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    pseudo_type: String,
    element: v8::Local<'_, v8::Object>,
    parent: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let pseudo = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, pseudo, prototype.into()) != Some(true) {
        return Err("cannot create CSSPseudoElement".to_owned());
    }
    let record = CssPseudoElementRecord {
        pseudo_type,
        element: v8::Global::new(scope, element),
        parent: v8::Global::new(scope, parent),
    };
    scope
        .get_slot_mut::<CssPseudoElementStore>()
        .ok_or_else(|| "CSSPseudoElement state was not prepared".to_owned())?
        .records
        .insert(pseudo.get_identity_hash().get(), record);
    Ok(pseudo)
}

pub(crate) fn valid_type(value: &str) -> bool {
    matches!(
        value,
        "::after"
            | "::backdrop"
            | "::before"
            | "::checkmark"
            | "::details-content"
            | "::file-selector-button"
            | "::first-letter"
            | "::first-line"
            | "::grammar-error"
            | "::marker"
            | "::picker-icon"
            | "::placeholder"
            | "::selection"
            | "::spelling-error"
    )
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssPseudoElementRecord> {
    scope
        .get_slot::<CssPseudoElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CSSPseudoElement': Illegal constructor",
    );
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.pseudo_type) {
        result.set(value.into());
    }
}

fn get_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.element).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_parent(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.parent).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn pseudo(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'pseudo' on 'CSSPseudoElement': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let pseudo_type = crate::webidl::value_to_string(scope, arguments.get(0));
    if !valid_type(&pseudo_type) {
        result.set(v8::null(scope).into());
        return;
    }
    let element = v8::Local::new(scope, &record.element);
    match create(scope, pseudo_type, element, arguments.this()) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
