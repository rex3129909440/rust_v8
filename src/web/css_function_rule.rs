use std::collections::HashMap;

#[derive(Clone)]
struct FunctionParameter {
    name: String,
    parameter_type: String,
    default_value: Option<String>,
}

#[derive(Clone)]
struct CssFunctionRuleRecord {
    name: String,
    return_type: String,
    parameters: Vec<FunctionParameter>,
}

#[derive(Default)]
pub(crate) struct CssFunctionRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssFunctionRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssFunctionRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSFunctionRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssFunctionRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSFunctionRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "returnType", get_return_type)?;
    crate::webidl::define_method(scope, prototype, "getParameters", 0, get_parameters)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_grouping_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssFunctionRuleStore>()
        .ok_or_else(|| "CSSFunctionRule state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    header: &str,
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let (name, parameters, return_type) = parse_header(header)?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let rule = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, rule, prototype.into()) != Some(true) {
        return Err("cannot create CSSFunctionRule".to_owned());
    }
    let declarations =
        super::css_function_declarations::create(scope, body, parent_style_sheet, Some(rule))?;
    super::css_grouping_rule::attach(scope, rule, vec![declarations])?;
    scope
        .get_slot_mut::<CssFunctionRuleStore>()
        .ok_or_else(|| "CSSFunctionRule state was not prepared".to_owned())?
        .records
        .insert(
            rule.get_identity_hash().get(),
            CssFunctionRuleRecord {
                name,
                return_type,
                parameters,
            },
        );
    let css_text = serialize(scope, rule).unwrap_or_default();
    super::css_rule::attach(scope, rule, 0, css_text, parent_style_sheet, parent_rule);
    Ok(rule)
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let mut parameters = String::new();
    for (index, parameter) in record.parameters.iter().enumerate() {
        if index > 0 {
            parameters.push_str(", ");
        }
        parameters.push_str(&parameter.name);
        parameters.push(' ');
        parameters.push_str(&parameter.parameter_type);
        if let Some(default_value) = &parameter.default_value {
            parameters.push_str(": ");
            parameters.push_str(default_value);
        }
    }
    let return_clause = if record.return_type == "*" {
        String::new()
    } else {
        format!(" returns {}", record.return_type)
    };
    let children = super::css_grouping_rule::list(scope, object)?;
    let children = v8::Local::new(scope, &children);
    let rules = super::css_rule_list::rules(scope, children)?;
    let body = rules
        .first()
        .map(|rule| v8::Local::new(scope, rule))
        .and_then(|rule| super::css_function_declarations::serialize(scope, rule))
        .unwrap_or_default();
    Some(format!(
        "@function {}({}){} {{ {} }}",
        record.name, parameters, return_clause, body
    ))
}

fn parse_header(header: &str) -> Result<(String, Vec<FunctionParameter>, String), String> {
    let rest = header["@function".len()..].trim();
    let open = rest
        .find('(')
        .ok_or_else(|| "CSS function is missing parameters".to_owned())?;
    let close = rest
        .rfind(')')
        .ok_or_else(|| "CSS function parameters are not closed".to_owned())?;
    let name = rest[..open].trim().to_owned();
    if !name.starts_with("--") {
        return Err("CSS function name must start with --".to_owned());
    }
    let mut parameters = Vec::new();
    for entry in rest[open + 1..close].split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let (definition, default_value) = entry
            .split_once(':')
            .map(|(definition, value)| (definition.trim(), Some(value.trim().to_owned())))
            .unwrap_or((entry, None));
        let mut pieces = definition.split_ascii_whitespace();
        let parameter_name = pieces.next().unwrap_or_default().to_owned();
        let parameter_type = pieces.collect::<Vec<_>>().join(" ");
        parameters.push(FunctionParameter {
            name: parameter_name,
            parameter_type: if parameter_type.is_empty() {
                "*".to_owned()
            } else {
                parameter_type
            },
            default_value,
        });
    }
    let after = rest[close + 1..].trim();
    let return_type = after
        .strip_prefix("returns")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("*")
        .to_owned();
    Ok((name, parameters, return_type))
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssFunctionRuleRecord> {
    scope
        .get_slot::<CssFunctionRuleStore>()?
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
        "Failed to construct 'CSSFunctionRule': Illegal constructor",
    );
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.name) {
        result.set(value.into());
    }
}

fn get_return_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.return_type) {
        result.set(value.into());
    }
}

fn get_parameters(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = v8::Array::new(scope, record.parameters.len() as i32);
    for (index, parameter) in record.parameters.iter().enumerate() {
        let value = v8::Object::new(scope);
        let _ = define_text(scope, value, "name", &parameter.name);
        let _ = define_text(scope, value, "type", &parameter.parameter_type);
        if let Some(default_value) = &parameter.default_value {
            let _ = define_text(scope, value, "defaultValue", default_value);
        }
        let _ = values.set_index(scope, index as u32, value.into());
    }
    result.set(values.into());
}

fn define_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let value = crate::webidl::string(scope, value)?;
    if object.define_own_property(scope, key.into(), value.into(), v8::PropertyAttribute::NONE)
        == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define CSS function parameter".to_owned())
    }
}
