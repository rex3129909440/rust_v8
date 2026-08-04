use std::collections::HashMap;

#[derive(Clone)]
struct CssKeyframesRuleRecord {
    name: String,
    rules: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct CssKeyframesRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssKeyframesRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssKeyframesRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSKeyframesRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssKeyframesRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSKeyframesRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "name", get_name, set_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "cssRules", get_css_rules)?;
    crate::webidl::define_method(scope, prototype, "appendRule", 1, append_rule)?;
    crate::webidl::define_method(scope, prototype, "deleteRule", 1, delete_rule)?;
    crate::webidl::define_method(scope, prototype, "findRule", 1, find_rule)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let iterator = crate::webidl::create_function(
        scope,
        "values",
        0,
        v8::ConstructorBehavior::Throw,
        iterator,
    )?;
    if prototype.define_own_property(
        scope,
        v8::Symbol::get_iterator(scope).into(),
        iterator.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define CSSKeyframesRule iterator".to_owned());
    }
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssKeyframesRuleStore>()
        .ok_or_else(|| "CSSKeyframesRule state was not prepared".to_owned())?
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

fn split_frames(body: &str) -> Result<Vec<(String, String)>, String> {
    let mut output = Vec::new();
    let bytes = body.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }
        let start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'{' {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            return Err("Keyframe rule is missing a block".to_owned());
        }
        let key = body[start..cursor].trim().to_owned();
        cursor += 1;
        let declarations_start = cursor;
        let mut depth = 1_u32;
        let mut quote = None;
        while cursor < bytes.len() && depth > 0 {
            let character = bytes[cursor] as char;
            if let Some(current) = quote {
                if character == current && bytes[cursor.saturating_sub(1)] != b'\\' {
                    quote = None;
                }
            } else if character == '\'' || character == '"' {
                quote = Some(character);
            } else if character == '{' {
                depth += 1;
            } else if character == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            cursor += 1;
        }
        if depth != 0 {
            return Err("Keyframe rule has an unterminated block".to_owned());
        }
        output.push((key, body[declarations_start..cursor].trim().to_owned()));
        cursor += 1;
    }
    Ok(output)
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
        return Err("cannot create CSSKeyframesRule".to_owned());
    }
    let mut frames = Vec::new();
    for (key, declarations) in split_frames(body)? {
        frames.push(super::css_keyframe_rule::create(
            scope,
            &key,
            &declarations,
            parent_style_sheet,
            Some(object),
        )?);
    }
    let rules = super::css_rule_list::create(scope, frames)?;
    super::css_rule::attach(
        scope,
        object,
        7,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    let rules = v8::Global::new(scope, rules);
    scope
        .get_slot_mut::<CssKeyframesRuleStore>()
        .ok_or_else(|| "CSSKeyframesRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssKeyframesRuleRecord { name, rules },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssKeyframesRuleRecord> {
    scope
        .get_slot::<CssKeyframesRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(name) = v8::String::new(scope, &record.name)
    {
        result.set(name.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    if name.trim().is_empty() {
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<CssKeyframesRuleStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.name = name;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_css_rules(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.rules).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let rules = v8::Local::new(scope, &record.rules);
    let length = super::css_rule_list::rules(scope, rules)
        .map(|rules| rules.len())
        .unwrap_or(0);
    result.set(v8::Integer::new_from_unsigned(scope, length as u32).into());
}

fn append_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(mut parsed) = split_frames(&text) else {
        return;
    };
    if parsed.len() != 1 {
        return;
    }
    let (key, declarations) = parsed.remove(0);
    let base = super::css_rule::record(scope, arguments.this());
    let parent_sheet = base
        .and_then(|record| record.parent_style_sheet)
        .map(|sheet| v8::Local::new(scope, &sheet));
    let frame = match super::css_keyframe_rule::create(
        scope,
        &key,
        &declarations,
        parent_sheet,
        Some(arguments.this()),
    ) {
        Ok(frame) => frame,
        Err(_) => return,
    };
    let list = v8::Local::new(scope, &record.rules);
    let length = super::css_rule_list::rules(scope, list)
        .map(|rules| rules.len())
        .unwrap_or(0);
    super::css_rule_list::insert(scope, list, length, frame);
}

fn requested_key(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> String {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value.eq_ignore_ascii_case("from") {
        "0%".to_owned()
    } else if value.eq_ignore_ascii_case("to") {
        "100%".to_owned()
    } else {
        value.trim().to_owned()
    }
}

fn delete_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let key = requested_key(scope, &arguments);
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(scope, &record.rules);
    let Some(rules) = super::css_rule_list::rules(scope, list) else {
        return;
    };
    for (index, rule) in rules.iter().enumerate() {
        if super::css_keyframe_rule::key_text(scope, v8::Local::new(scope, rule)).as_deref()
            == Some(key.as_str())
        {
            super::css_rule_list::delete(scope, list, index);
            return;
        }
    }
}

fn find_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let key = requested_key(scope, &arguments);
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(scope, &record.rules);
    if let Some(rules) = super::css_rule_list::rules(scope, list) {
        for rule in rules {
            let local = v8::Local::new(scope, &rule);
            if super::css_keyframe_rule::key_text(scope, local).as_deref() == Some(key.as_str()) {
                result.set(local.into());
                return;
            }
        }
    }
    result.set(v8::null(scope).into());
}

fn iterator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(scope, &record.rules);
    let key = v8::Symbol::get_iterator(scope);
    let function = list
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok());
    if let Some(function) = function
        && let Some(iterator) = function.call(scope, list.into(), &[])
    {
        result.set(iterator);
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let rules = super::css_rule_list::rules(scope, v8::Local::new(scope, &record.rules))?;
    let mut body = String::new();
    for rule in rules {
        if let Some(text) = super::css_keyframe_rule::serialize(scope, v8::Local::new(scope, &rule))
        {
            if !body.is_empty() {
                body.push(' ');
            }
            body.push_str(&text);
        }
    }
    Some(format!("@keyframes {} {{ {} }}", record.name, body))
}
