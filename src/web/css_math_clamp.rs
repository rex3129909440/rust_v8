use std::collections::HashMap;

#[derive(Clone)]
struct CssMathClampRecord {
    lower: v8::Global<v8::Object>,
    value: v8::Global<v8::Object>,
    upper: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssMathClampStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssMathClampRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssMathClampStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSMathClamp", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssMathClampStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSMathClamp",
        3,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lower", get_lower)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "value", get_value)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "upper", get_upper)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_math_value::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssMathClampStore>()
        .ok_or_else(|| "CSSMathClamp state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 3 {
        crate::webidl::throw_type_error(scope, "CSSMathClamp requires three values");
        return;
    }
    let lower = match super::css_numeric_value::numberish(scope, arguments.get(0)) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let value = match super::css_numeric_value::numberish(scope, arguments.get(1)) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let upper = match super::css_numeric_value::numberish(scope, arguments.get(2)) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let compatibility_values = vec![lower.clone(), value.clone(), upper.clone()];
    if !super::css_numeric_value::compatible(scope, &compatibility_values) {
        crate::webidl::throw_type_error(scope, "Incompatible types");
        return;
    }
    let record = CssMathClampRecord {
        lower,
        value,
        upper,
    };
    scope
        .get_slot_mut::<CssMathClampStore>()
        .expect("CSSMathClamp state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    super::css_math_value::attach(scope, arguments.this(), "clamp");
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssMathClampRecord> {
    scope
        .get_slot::<CssMathClampStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_lower(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.lower).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_upper(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.upper).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let lower = super::css_style_value::serialize(scope, v8::Local::new(scope, &record.lower))?;
    let value = super::css_style_value::serialize(scope, v8::Local::new(scope, &record.value))?;
    let upper = super::css_style_value::serialize(scope, v8::Local::new(scope, &record.upper))?;
    Some(format!("clamp({lower}, {value}, {upper})"))
}
