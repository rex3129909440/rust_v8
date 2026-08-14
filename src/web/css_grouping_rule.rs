use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssGroupingRuleStore {
    constructor: crate::webidl::RealmConstructor,
    rule_lists: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssGroupingRuleStore::default());
}

#[allow(dead_code)]
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSGroupingRule", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssGroupingRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSGroupingRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "cssRules", get_css_rules)?;
    crate::webidl::define_method(scope, prototype, "deleteRule", 1, delete_rule)?;
    crate::webidl::define_method(scope, prototype, "insertRule", 1, insert_rule)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssGroupingRuleStore>()
        .ok_or_else(|| "CSSGroupingRule state was not prepared".to_owned())?
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

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    rules: Vec<v8::Local<'_, v8::Object>>,
) -> Result<(), String> {
    let list = super::css_rule_list::create(scope, rules)?;
    let list = v8::Global::new(scope, list);
    scope
        .get_slot_mut::<CssGroupingRuleStore>()
        .ok_or_else(|| "CSSGroupingRule state was not prepared".to_owned())?
        .rule_lists
        .insert(object.get_identity_hash().get(), list);
    Ok(())
}

pub(crate) fn list(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<CssGroupingRuleStore>()?
        .rule_lists
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn replace_rules(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    rules: Vec<v8::Local<'_, v8::Object>>,
) -> bool {
    let Some(list) = list(scope, object) else {
        return false;
    };
    let list = v8::Local::new(scope, &list);
    super::css_rule_list::replace(scope, list, rules)
}

pub(crate) fn serialized_body(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let list = list(scope, object)?;
    let rules = super::css_rule_list::rules(scope, v8::Local::new(scope, &list))?;
    let mut body = String::new();
    for rule in rules {
        let text = super::css_rule::serialized(scope, v8::Local::new(scope, &rule))?;
        for line in text.lines() {
            body.push_str("  ");
            body.push_str(line);
            body.push('\n');
        }
    }
    Some(body)
}

fn get_css_rules(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(list) = list(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &list).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn delete_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    let Some(list) = list(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(scope, &list);
    if !super::css_rule_list::delete(scope, list, index) {
        crate::webidl::throw_type_error(scope, "Rule index is outside the list");
    }
}

pub(crate) fn insert_rule(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let text = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(list_global) = list(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(scope, &list_global);
    let length = super::css_rule_list::rules(scope, list)
        .map(|rules| rules.len())
        .unwrap_or(0);
    let index = if arguments.get(1).is_undefined() {
        length
    } else {
        arguments.get(1).uint32_value(scope).unwrap_or(u32::MAX) as usize
    };
    let parent = super::css_rule::record(scope, arguments.this());
    let parent_sheet = parent
        .as_ref()
        .and_then(|record| record.parent_style_sheet.as_ref())
        .cloned()
        .map(|sheet| v8::Local::new(scope, &sheet));
    let rule = match super::css_style_sheet::parse_single_rule(
        scope,
        &text,
        parent_sheet,
        Some(arguments.this()),
    ) {
        Ok(rule) => rule,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if super::css_rule_list::insert(scope, list, index, rule) {
        result.set(v8::Integer::new_from_unsigned(scope, index as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Rule index is outside the list");
    }
}
