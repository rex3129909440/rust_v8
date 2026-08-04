use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CssUnitRecord {
    pub value: f64,
    pub unit: String,
}

#[derive(Default)]
pub(crate) struct CssUnitValueStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssUnitRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssUnitValueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSUnitValue", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssUnitValueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSUnitValue",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "value", get_value, set_value)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "unit", get_unit)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_numeric_value::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssUnitValueStore>()
        .ok_or_else(|| "CSSUnitValue state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "CSSUnitValue requires a value and unit");
        return;
    }
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    let unit = crate::webidl::value_to_string(scope, arguments.get(1)).to_ascii_lowercase();
    if unit.is_empty() || !value.is_finite() {
        crate::webidl::throw_type_error(scope, "Invalid CSS numeric value");
        return;
    }
    attach(scope, arguments.this(), value, unit);
    result.set(arguments.this().into());
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: f64,
    unit: String,
) {
    let unit = if unit == "%" {
        "percent".to_owned()
    } else {
        unit
    };
    if let Some(store) = scope.get_slot_mut::<CssUnitValueStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            CssUnitRecord { value, unit },
        );
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: f64,
    unit: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSUnitValue".to_owned());
    }
    attach(scope, object, value, unit.to_owned());
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssUnitRecord> {
    scope
        .get_slot::<CssUnitValueStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "CSS unit value must be finite");
        return;
    }
    if let Some(record) = scope.get_slot_mut::<CssUnitValueStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.value = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_unit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(unit) = v8::String::new(scope, &record.unit) {
            result.set(unit.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(record: &CssUnitRecord) -> String {
    if record.unit == "number" {
        format!("{}", record.value)
    } else if record.unit == "percent" {
        format!("{}%", record.value)
    } else {
        format!("{}{}", record.value, record.unit)
    }
}
