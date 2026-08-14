use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct CssSupportsRuleStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssSupportsRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSSupportsRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssSupportsRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSSupportsRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_condition_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssSupportsRuleStore>()
        .ok_or_else(|| "CSSSupportsRule state was not prepared".to_owned())?
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
    condition: String,
    body: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSSupportsRule".to_owned());
    }
    super::css_grouping_rule::attach(scope, object, Vec::new())?;
    super::css_condition_rule::attach(scope, object, condition);
    super::css_rule::attach(
        scope,
        object,
        12,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    scope
        .get_slot_mut::<CssSupportsRuleStore>()
        .ok_or_else(|| "CSSSupportsRule state was not prepared".to_owned())?
        .objects
        .insert(object.get_identity_hash().get());
    let nested =
        super::css_style_sheet::parse_rules(scope, body, parent_style_sheet, Some(object))?;
    super::css_grouping_rule::replace_rules(scope, object, nested);
    Ok(object)
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    if !scope
        .get_slot::<CssSupportsRuleStore>()?
        .objects
        .contains(&object.get_identity_hash().get())
    {
        return None;
    }
    let condition = super::css_condition_rule::condition(scope, object)?;
    let body = super::css_grouping_rule::serialized_body(scope, object)?;
    Some(format!("@supports {condition} {{\n{body}}}"))
}
