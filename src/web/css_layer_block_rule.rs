use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssLayerBlockRuleStore {
    constructor: crate::webidl::RealmConstructor,
    names: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssLayerBlockRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSLayerBlockRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssLayerBlockRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSLayerBlockRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_grouping_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssLayerBlockRuleStore>()
        .ok_or_else(|| "CSSLayerBlockRule state was not prepared".to_owned())?
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
        return Err("cannot create CSSLayerBlockRule".to_owned());
    }
    super::css_grouping_rule::attach(scope, object, Vec::new())?;
    super::css_rule::attach(
        scope,
        object,
        0,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    scope
        .get_slot_mut::<CssLayerBlockRuleStore>()
        .ok_or_else(|| "CSSLayerBlockRule state was not prepared".to_owned())?
        .names
        .insert(object.get_identity_hash().get(), name);
    let nested =
        super::css_style_sheet::parse_rules(scope, body, parent_style_sheet, Some(object))?;
    super::css_grouping_rule::replace_rules(scope, object, nested);
    Ok(object)
}

fn name(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<String> {
    scope
        .get_slot::<CssLayerBlockRuleStore>()?
        .names
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(name) = name(scope, arguments.this())
        && let Some(name) = v8::String::new(scope, &name)
    {
        result.set(name.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let name = name(scope, object)?;
    let list = super::css_grouping_rule::list(scope, object)?;
    let rules = super::css_rule_list::rules(scope, v8::Local::new(scope, &list))?;
    let mut body = String::new();
    for rule in rules {
        let rule = v8::Local::new(scope, &rule);
        if let Some(text) = super::css_rule::serialized(scope, rule) {
            if !body.is_empty() {
                body.push(' ');
            }
            body.push_str(&text);
        }
    }
    Some(format!("@layer {name} {{ {body} }}"))
}
