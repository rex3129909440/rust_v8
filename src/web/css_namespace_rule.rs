use std::collections::HashMap;

#[derive(Clone)]
struct CssNamespaceRuleRecord {
    namespace_uri: String,
    prefix: String,
}

#[derive(Default)]
pub(crate) struct CssNamespaceRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssNamespaceRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssNamespaceRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSNamespaceRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssNamespaceRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSNamespaceRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "namespaceURI", get_namespace_uri)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "prefix", get_prefix)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssNamespaceRuleStore>()
        .ok_or_else(|| "CSSNamespaceRule state was not prepared".to_owned())?
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
    prefix: String,
    namespace_uri: String,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSNamespaceRule".to_owned());
    }
    super::css_rule::attach(scope, object, 10, String::new(), parent_style_sheet, None);
    scope
        .get_slot_mut::<CssNamespaceRuleStore>()
        .ok_or_else(|| "CSSNamespaceRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssNamespaceRuleRecord {
                namespace_uri,
                prefix,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssNamespaceRuleRecord> {
    scope
        .get_slot::<CssNamespaceRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_namespace_uri(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.namespace_uri) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_prefix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.prefix) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let prefix = if record.prefix.is_empty() {
        String::new()
    } else {
        format!("{} ", record.prefix)
    };
    Some(format!(
        "@namespace {}url(\"{}\");",
        prefix, record.namespace_uri
    ))
}
