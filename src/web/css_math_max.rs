use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssMathMaxStore {
    constructor: crate::webidl::RealmConstructor,
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssMathMaxStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSMathMax", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssMathMaxStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSMathMax",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "values", get_values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_math_value::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssMathMaxStore>()
        .ok_or_else(|| "CSSMathMax state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "CSSMathMax arguments cannot be empty");
        return;
    }
    let mut stored_members = Vec::with_capacity(arguments.length() as usize);
    for index in 0..arguments.length() {
        let member = match super::css_numeric_value::numberish(scope, arguments.get(index)) {
            Ok(member) => member,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
        stored_members.push(member);
    }
    if !super::css_numeric_value::compatible(scope, &stored_members) {
        crate::webidl::throw_type_error(scope, "Incompatible types");
        return;
    }
    let mut members = Vec::with_capacity(stored_members.len());
    for member in &stored_members {
        members.push(v8::Local::new(scope, member));
    }
    let values = match super::css_numeric_array::create(scope, &members) {
        Ok(values) => values,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let values = v8::Global::new(scope, values);
    scope
        .get_slot_mut::<CssMathMaxStore>()
        .expect("CSSMathMax state")
        .values
        .insert(arguments.this().get_identity_hash().get(), values);
    super::css_math_value::attach(scope, arguments.this(), "max");
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<CssMathMaxStore>()?
        .values
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &values).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let values = record(scope, object)?;
    let values = v8::Local::new(scope, &values);
    let length_key = v8::String::new(scope, "length")?;
    let length = values.get(scope, length_key.into())?.uint32_value(scope)?;
    let mut output = String::from("max(");
    for index in 0..length {
        let value = v8::Local::<v8::Object>::try_from(values.get_index(scope, index)?).ok()?;
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&super::css_style_value::serialize(scope, value)?);
    }
    output.push(')');
    Some(output)
}
