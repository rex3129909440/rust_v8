use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CssVariableRecord {
    pub variable: String,
    pub fallback: Option<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct CssVariableReferenceValueStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssVariableRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssVariableReferenceValueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSVariableReferenceValue", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssVariableReferenceValueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSVariableReferenceValue",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "variable", get_variable, set_variable)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "fallback", get_fallback)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssVariableReferenceValueStore>()
        .ok_or_else(|| "CSSVariableReferenceValue state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CSSVariableReferenceValue requires a variable");
        return;
    }
    let variable = crate::webidl::value_to_string(scope, arguments.get(0));
    if !variable.starts_with("--") {
        crate::webidl::throw_type_error(scope, "Variable names must start with '--'");
        return;
    }
    let fallback = if arguments.get(1).is_null_or_undefined() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(arguments.get(1))
            .ok()
            .map(|fallback| v8::Global::new(scope, fallback))
    };
    scope
        .get_slot_mut::<CssVariableReferenceValueStore>()
        .expect("CSSVariableReferenceValue state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CssVariableRecord { variable, fallback },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssVariableRecord> {
    scope
        .get_slot::<CssVariableReferenceValueStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_variable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(variable) = v8::String::new(scope, &record.variable) {
            result.set(variable.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_variable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let variable = crate::webidl::value_to_string(scope, arguments.get(0));
    if !variable.starts_with("--") {
        crate::webidl::throw_type_error(scope, "Variable names must start with '--'");
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<CssVariableReferenceValueStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.variable = variable;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_fallback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.fallback {
            Some(fallback) => result.set(v8::Local::new(scope, &fallback).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
