use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssFunctionDeclarationsStore {
    constructor: crate::webidl::RealmConstructor,
    styles: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssFunctionDeclarationsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSFunctionDeclarations", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssFunctionDeclarationsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSFunctionDeclarations",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "style", get_style, set_style)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssFunctionDeclarationsStore>()
        .ok_or_else(|| "CSSFunctionDeclarations state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    declarations: &str,
    parent_style_sheet: Option<v8::Local<'_, v8::Object>>,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let rule = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, rule, prototype.into()) != Some(true) {
        return Err("cannot create CSSFunctionDeclarations".to_owned());
    }
    let style = super::css_function_descriptors::create(scope, declarations, Some(rule))?;
    let style = v8::Global::new(scope, style);
    scope
        .get_slot_mut::<CssFunctionDeclarationsStore>()
        .ok_or_else(|| "CSSFunctionDeclarations state was not prepared".to_owned())?
        .styles
        .insert(rule.get_identity_hash().get(), style);
    super::css_rule::attach(
        scope,
        rule,
        0,
        declarations.to_owned(),
        parent_style_sheet,
        parent_rule,
    );
    Ok(rule)
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let style = scope
        .get_slot::<CssFunctionDeclarationsStore>()?
        .styles
        .get(&object.get_identity_hash().get())?
        .clone();
    super::css_style_declaration::css_text(scope, v8::Local::new(scope, &style))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CSSFunctionDeclarations': Illegal constructor",
    );
}

fn get_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let style = scope
        .get_slot::<CssFunctionDeclarationsStore>()
        .and_then(|store| {
            store
                .styles
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(style) = style {
        result.set(v8::Local::new(scope, &style).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_style(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(style) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        return;
    };
    if super::css_style_declaration::css_text(scope, style).is_none() {
        return;
    }
    let style = v8::Global::new(scope, style);
    let Some(current) = scope
        .get_slot_mut::<CssFunctionDeclarationsStore>()
        .and_then(|store| {
            store
                .styles
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    *current = style;
}
