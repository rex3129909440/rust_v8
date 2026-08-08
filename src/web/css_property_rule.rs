use std::collections::HashMap;

#[derive(Clone)]
struct CssPropertyRuleRecord {
    name: String,
    syntax: String,
    inherits: bool,
    initial_value: Option<String>,
}

#[derive(Default)]
pub(crate) struct CssPropertyRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssPropertyRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssPropertyRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSPropertyRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssPropertyRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSPropertyRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "syntax", get_syntax)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "inherits", get_inherits)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "initialValue", get_initial_value)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssPropertyRuleStore>()
        .ok_or_else(|| "CSSPropertyRule state was not prepared".to_owned())?
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
    name: String,
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSPropertyRule".to_owned());
    }
    let declarations = super::css_style_declaration::parse_declarations(body);
    let syntax = declarations
        .iter()
        .find(|property| property.name == "syntax")
        .map(|property| property.value.trim_matches(['\'', '"']).to_owned())
        .unwrap_or("*".to_owned());
    let inherits = declarations
        .iter()
        .find(|property| property.name == "inherits")
        .is_some_and(|property| property.value.eq_ignore_ascii_case("true"));
    let initial_value = declarations
        .iter()
        .find(|property| property.name == "initial-value")
        .map(|property| property.value.clone());
    super::css_rule::attach(
        scope,
        object,
        0,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    scope
        .get_slot_mut::<CssPropertyRuleStore>()
        .ok_or_else(|| "CSSPropertyRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssPropertyRuleRecord {
                name,
                syntax,
                inherits,
                initial_value,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssPropertyRuleRecord> {
    scope
        .get_slot::<CssPropertyRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn string_result(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<&str>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        if let Some(value) = v8::String::new(scope, value) {
            result.set(value.into());
        }
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        string_result(s, Some(&v.name), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_syntax(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        string_result(s, Some(&v.syntax), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_inherits(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.inherits).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_initial_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        string_result(s, v.initial_value.as_deref(), r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let mut descriptors = format!(
        "syntax: \"{}\"; inherits: {};",
        record.syntax, record.inherits
    );
    if let Some(initial) = record.initial_value {
        descriptors.push_str(" initial-value: ");
        descriptors.push_str(&initial);
        descriptors.push(';');
    }
    Some(format!("@property {} {{ {} }}", record.name, descriptors))
}
