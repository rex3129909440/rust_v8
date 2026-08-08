use std::collections::HashMap;

#[derive(Clone)]
struct CssFontPaletteValuesRuleRecord {
    name: String,
    font_family: String,
    base_palette: String,
    override_colors: String,
}

#[derive(Default)]
pub(crate) struct CssFontPaletteValuesRuleStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CssFontPaletteValuesRuleRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssFontPaletteValuesRuleStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSFontPaletteValuesRule", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssFontPaletteValuesRuleStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSFontPaletteValuesRule",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "fontFamily", get_font_family)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "basePalette", get_base_palette)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "overrideColors",
        get_override_colors,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_rule::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssFontPaletteValuesRuleStore>()
        .ok_or_else(|| "CSSFontPaletteValuesRule state was not prepared".to_owned())?
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
        return Err("cannot create CSSFontPaletteValuesRule".to_owned());
    }
    let declarations = super::css_style_declaration::create(scope, body, Some(object), None)?;
    let font_family = super::css_style_declaration::named_value(scope, declarations, "font-family")
        .unwrap_or_default();
    let base_palette =
        super::css_style_declaration::named_value(scope, declarations, "base-palette")
            .unwrap_or_default();
    let override_colors =
        super::css_style_declaration::named_value(scope, declarations, "override-colors")
            .unwrap_or_default();
    super::css_rule::attach(
        scope,
        object,
        0,
        String::new(),
        parent_style_sheet,
        parent_rule,
    );
    scope
        .get_slot_mut::<CssFontPaletteValuesRuleStore>()
        .ok_or_else(|| "CSSFontPaletteValuesRule state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CssFontPaletteValuesRuleRecord {
                name,
                font_family,
                base_palette,
                override_colors,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CssFontPaletteValuesRuleRecord> {
    scope
        .get_slot::<CssFontPaletteValuesRuleStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<String>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value
        && let Some(value) = v8::String::new(scope, &value)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(
        scope,
        record(scope, arguments.this()).map(|record| record.name),
        result,
    );
}

fn get_font_family(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(
        scope,
        record(scope, arguments.this()).map(|record| record.font_family),
        result,
    );
}

fn get_base_palette(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(
        scope,
        record(scope, arguments.this()).map(|record| record.base_palette),
        result,
    );
}

fn get_override_colors(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(
        scope,
        record(scope, arguments.this()).map(|record| record.override_colors),
        result,
    );
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let record = record(scope, object)?;
    let mut declarations = String::new();
    if !record.font_family.is_empty() {
        declarations.push_str(&format!("font-family: {}; ", record.font_family));
    }
    if !record.base_palette.is_empty() {
        declarations.push_str(&format!("base-palette: {}; ", record.base_palette));
    }
    if !record.override_colors.is_empty() {
        declarations.push_str(&format!("override-colors: {}; ", record.override_colors));
    }
    Some(format!(
        "@font-palette-values {} {{ {} }}",
        record.name,
        declarations.trim_end()
    ))
}
