#[derive(Default)]
pub(crate) struct CssPositionTryDescriptorsStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssPositionTryDescriptorsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSPositionTryDescriptors", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssPositionTryDescriptorsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSPositionTryDescriptors",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    define_surface(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_style_declaration::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssPositionTryDescriptorsStore>()
        .ok_or_else(|| "CSSPositionTryDescriptors state was not prepared".to_owned())?
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
    declarations: &str,
    parent_rule: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = super::css_style_declaration::create(scope, declarations, parent_rule, None)?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSPositionTryDescriptors".to_owned());
    }
    Ok(object)
}

fn get_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = super::css_style_declaration::named_value(scope, object, name) {
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) {
    let value = crate::webidl::value_to_string(scope, value);
    if !super::css_style_declaration::set_named_value(scope, object, name, value) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

macro_rules! property_callbacks {
    ($getter:ident, $setter:ident, $name:literal) => {
        fn $getter(
            scope: &mut v8::PinScope<'_, '_>,
            arguments: v8::FunctionCallbackArguments<'_>,
            result: v8::ReturnValue<'_>,
        ) {
            get_property(scope, arguments.this(), $name, result);
        }
        fn $setter(
            scope: &mut v8::PinScope<'_, '_>,
            arguments: v8::FunctionCallbackArguments<'_>,
            _: v8::ReturnValue<'_>,
        ) {
            set_property(scope, arguments.this(), arguments.get(0), $name);
        }
    };
}

property_callbacks!(get_margin, set_margin, "margin");
property_callbacks!(get_margin_top, set_margin_top, "margin-top");
property_callbacks!(get_margin_right, set_margin_right, "margin-right");
property_callbacks!(get_margin_bottom, set_margin_bottom, "margin-bottom");
property_callbacks!(get_margin_left, set_margin_left, "margin-left");
property_callbacks!(get_margin_block, set_margin_block, "margin-block");
property_callbacks!(
    get_margin_block_start,
    set_margin_block_start,
    "margin-block-start"
);
property_callbacks!(
    get_margin_block_end,
    set_margin_block_end,
    "margin-block-end"
);
property_callbacks!(get_margin_inline, set_margin_inline, "margin-inline");
property_callbacks!(
    get_margin_inline_start,
    set_margin_inline_start,
    "margin-inline-start"
);
property_callbacks!(
    get_margin_inline_end,
    set_margin_inline_end,
    "margin-inline-end"
);
property_callbacks!(get_inset, set_inset, "inset");
property_callbacks!(get_inset_block, set_inset_block, "inset-block");
property_callbacks!(
    get_inset_block_start,
    set_inset_block_start,
    "inset-block-start"
);
property_callbacks!(get_inset_block_end, set_inset_block_end, "inset-block-end");
property_callbacks!(get_inset_inline, set_inset_inline, "inset-inline");
property_callbacks!(
    get_inset_inline_start,
    set_inset_inline_start,
    "inset-inline-start"
);
property_callbacks!(
    get_inset_inline_end,
    set_inset_inline_end,
    "inset-inline-end"
);
property_callbacks!(get_top, set_top, "top");
property_callbacks!(get_left, set_left, "left");
property_callbacks!(get_right, set_right, "right");
property_callbacks!(get_bottom, set_bottom, "bottom");
property_callbacks!(get_width, set_width, "width");
property_callbacks!(get_min_width, set_min_width, "min-width");
property_callbacks!(get_max_width, set_max_width, "max-width");
property_callbacks!(get_height, set_height, "height");
property_callbacks!(get_min_height, set_min_height, "min-height");
property_callbacks!(get_max_height, set_max_height, "max-height");
property_callbacks!(get_block_size, set_block_size, "block-size");
property_callbacks!(get_min_block_size, set_min_block_size, "min-block-size");
property_callbacks!(get_max_block_size, set_max_block_size, "max-block-size");
property_callbacks!(get_inline_size, set_inline_size, "inline-size");
property_callbacks!(get_min_inline_size, set_min_inline_size, "min-inline-size");
property_callbacks!(get_max_inline_size, set_max_inline_size, "max-inline-size");
property_callbacks!(get_place_self, set_place_self, "place-self");
property_callbacks!(get_align_self, set_align_self, "align-self");
property_callbacks!(get_justify_self, set_justify_self, "justify-self");
property_callbacks!(get_position_anchor, set_position_anchor, "position-anchor");
property_callbacks!(get_position_area, set_position_area, "position-area");

fn define_surface(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "margin", get_margin, set_margin)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginTop",
        get_margin_top,
        set_margin_top,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginRight",
        get_margin_right,
        set_margin_right,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginBottom",
        get_margin_bottom,
        set_margin_bottom,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginLeft",
        get_margin_left,
        set_margin_left,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginBlock",
        get_margin_block,
        set_margin_block,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginBlockStart",
        get_margin_block_start,
        set_margin_block_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginBlockEnd",
        get_margin_block_end,
        set_margin_block_end,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginInline",
        get_margin_inline,
        set_margin_inline,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginInlineStart",
        get_margin_inline_start,
        set_margin_inline_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "marginInlineEnd",
        get_margin_inline_end,
        set_margin_inline_end,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-top",
        get_margin_top,
        set_margin_top,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-right",
        get_margin_right,
        set_margin_right,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-bottom",
        get_margin_bottom,
        set_margin_bottom,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-left",
        get_margin_left,
        set_margin_left,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-block",
        get_margin_block,
        set_margin_block,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-block-start",
        get_margin_block_start,
        set_margin_block_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-block-end",
        get_margin_block_end,
        set_margin_block_end,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-inline",
        get_margin_inline,
        set_margin_inline,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-inline-start",
        get_margin_inline_start,
        set_margin_inline_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "margin-inline-end",
        get_margin_inline_end,
        set_margin_inline_end,
    )?;
    crate::webidl::define_accessor(scope, prototype, "inset", get_inset, set_inset)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "insetBlock",
        get_inset_block,
        set_inset_block,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "insetBlockStart",
        get_inset_block_start,
        set_inset_block_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "insetBlockEnd",
        get_inset_block_end,
        set_inset_block_end,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "insetInline",
        get_inset_inline,
        set_inset_inline,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "insetInlineStart",
        get_inset_inline_start,
        set_inset_inline_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "insetInlineEnd",
        get_inset_inline_end,
        set_inset_inline_end,
    )?;
    crate::webidl::define_accessor(scope, prototype, "top", get_top, set_top)?;
    crate::webidl::define_accessor(scope, prototype, "left", get_left, set_left)?;
    crate::webidl::define_accessor(scope, prototype, "right", get_right, set_right)?;
    crate::webidl::define_accessor(scope, prototype, "bottom", get_bottom, set_bottom)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inset-block",
        get_inset_block,
        set_inset_block,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inset-block-start",
        get_inset_block_start,
        set_inset_block_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inset-block-end",
        get_inset_block_end,
        set_inset_block_end,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inset-inline",
        get_inset_inline,
        set_inset_inline,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inset-inline-start",
        get_inset_inline_start,
        set_inset_inline_start,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inset-inline-end",
        get_inset_inline_end,
        set_inset_inline_end,
    )?;
    crate::webidl::define_accessor(scope, prototype, "width", get_width, set_width)?;
    crate::webidl::define_accessor(scope, prototype, "minWidth", get_min_width, set_min_width)?;
    crate::webidl::define_accessor(scope, prototype, "maxWidth", get_max_width, set_max_width)?;
    crate::webidl::define_accessor(scope, prototype, "height", get_height, set_height)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "minHeight",
        get_min_height,
        set_min_height,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "maxHeight",
        get_max_height,
        set_max_height,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "blockSize",
        get_block_size,
        set_block_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "minBlockSize",
        get_min_block_size,
        set_min_block_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "maxBlockSize",
        get_max_block_size,
        set_max_block_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inlineSize",
        get_inline_size,
        set_inline_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "minInlineSize",
        get_min_inline_size,
        set_min_inline_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "maxInlineSize",
        get_max_inline_size,
        set_max_inline_size,
    )?;
    crate::webidl::define_accessor(scope, prototype, "min-width", get_min_width, set_min_width)?;
    crate::webidl::define_accessor(scope, prototype, "max-width", get_max_width, set_max_width)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "min-height",
        get_min_height,
        set_min_height,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "max-height",
        get_max_height,
        set_max_height,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "block-size",
        get_block_size,
        set_block_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "min-block-size",
        get_min_block_size,
        set_min_block_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "max-block-size",
        get_max_block_size,
        set_max_block_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "inline-size",
        get_inline_size,
        set_inline_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "min-inline-size",
        get_min_inline_size,
        set_min_inline_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "max-inline-size",
        get_max_inline_size,
        set_max_inline_size,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "placeSelf",
        get_place_self,
        set_place_self,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "alignSelf",
        get_align_self,
        set_align_self,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "justifySelf",
        get_justify_self,
        set_justify_self,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "place-self",
        get_place_self,
        set_place_self,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "align-self",
        get_align_self,
        set_align_self,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "justify-self",
        get_justify_self,
        set_justify_self,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "positionAnchor",
        get_position_anchor,
        set_position_anchor,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "position-anchor",
        get_position_anchor,
        set_position_anchor,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "positionArea",
        get_position_area,
        set_position_area,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "position-area",
        get_position_area,
        set_position_area,
    )
}
