use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct CssNamespaceStore {
    object: Option<v8::Global<v8::Object>>,
    highlights: Option<v8::Global<v8::Object>>,
    paint_worklet: Option<v8::Global<v8::Object>>,
    registered_properties: HashSet<String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssNamespaceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let object = v8::Object::new(scope);
    let highlights = super::highlight_registry::create(scope)?;
    let paint_worklet = super::worklet::create(scope)?;
    crate::webidl::define_readonly_accessor(scope, object, "highlights", get_highlights)?;
    crate::webidl::define_method(scope, object, "Hz", 1, hz)?;
    crate::webidl::define_method(scope, object, "Q", 1, q)?;
    crate::webidl::define_method(scope, object, "cap", 1, cap)?;
    crate::webidl::define_method(scope, object, "ch", 1, ch)?;
    crate::webidl::define_method(scope, object, "cm", 1, cm)?;
    crate::webidl::define_method(scope, object, "cqb", 1, cqb)?;
    crate::webidl::define_method(scope, object, "cqh", 1, cqh)?;
    crate::webidl::define_method(scope, object, "cqi", 1, cqi)?;
    crate::webidl::define_method(scope, object, "cqmax", 1, cqmax)?;
    crate::webidl::define_method(scope, object, "cqmin", 1, cqmin)?;
    crate::webidl::define_method(scope, object, "cqw", 1, cqw)?;
    crate::webidl::define_method(scope, object, "deg", 1, deg)?;
    crate::webidl::define_method(scope, object, "dpcm", 1, dpcm)?;
    crate::webidl::define_method(scope, object, "dpi", 1, dpi)?;
    crate::webidl::define_method(scope, object, "dppx", 1, dppx)?;
    crate::webidl::define_method(scope, object, "dvb", 1, dvb)?;
    crate::webidl::define_method(scope, object, "dvh", 1, dvh)?;
    crate::webidl::define_method(scope, object, "dvi", 1, dvi)?;
    crate::webidl::define_method(scope, object, "dvmax", 1, dvmax)?;
    crate::webidl::define_method(scope, object, "dvmin", 1, dvmin)?;
    crate::webidl::define_method(scope, object, "dvw", 1, dvw)?;
    crate::webidl::define_method(scope, object, "em", 1, em)?;
    crate::webidl::define_method(scope, object, "escape", 1, escape)?;
    crate::webidl::define_method(scope, object, "ex", 1, ex)?;
    crate::webidl::define_method(scope, object, "fr", 1, fr)?;
    crate::webidl::define_method(scope, object, "grad", 1, grad)?;
    crate::webidl::define_method(scope, object, "ic", 1, ic)?;
    crate::webidl::define_method(scope, object, "in", 1, inch)?;
    crate::webidl::define_method(scope, object, "kHz", 1, khz)?;
    crate::webidl::define_method(scope, object, "lh", 1, lh)?;
    crate::webidl::define_method(scope, object, "lvb", 1, lvb)?;
    crate::webidl::define_method(scope, object, "lvh", 1, lvh)?;
    crate::webidl::define_method(scope, object, "lvi", 1, lvi)?;
    crate::webidl::define_method(scope, object, "lvmax", 1, lvmax)?;
    crate::webidl::define_method(scope, object, "lvmin", 1, lvmin)?;
    crate::webidl::define_method(scope, object, "lvw", 1, lvw)?;
    crate::webidl::define_method(scope, object, "mm", 1, mm)?;
    crate::webidl::define_method(scope, object, "ms", 1, ms)?;
    crate::webidl::define_method(scope, object, "number", 1, number)?;
    crate::webidl::define_method(scope, object, "pc", 1, pc)?;
    crate::webidl::define_method(scope, object, "percent", 1, percent)?;
    crate::webidl::define_method(scope, object, "pt", 1, pt)?;
    crate::webidl::define_method(scope, object, "px", 1, px)?;
    crate::webidl::define_method(scope, object, "rad", 1, rad)?;
    crate::webidl::define_method(scope, object, "rcap", 1, rcap)?;
    crate::webidl::define_method(scope, object, "rch", 1, rch)?;
    crate::webidl::define_method(scope, object, "registerProperty", 1, register_property)?;
    crate::webidl::define_method(scope, object, "rem", 1, rem)?;
    crate::webidl::define_method(scope, object, "rex", 1, rex)?;
    crate::webidl::define_method(scope, object, "ric", 1, ric)?;
    crate::webidl::define_method(scope, object, "rlh", 1, rlh)?;
    crate::webidl::define_method(scope, object, "s", 1, seconds)?;
    crate::webidl::define_method(scope, object, "supports", 1, supports)?;
    crate::webidl::define_method(scope, object, "svb", 1, svb)?;
    crate::webidl::define_method(scope, object, "svh", 1, svh)?;
    crate::webidl::define_method(scope, object, "svi", 1, svi)?;
    crate::webidl::define_method(scope, object, "svmax", 1, svmax)?;
    crate::webidl::define_method(scope, object, "svmin", 1, svmin)?;
    crate::webidl::define_method(scope, object, "svw", 1, svw)?;
    crate::webidl::define_method(scope, object, "turn", 1, turn)?;
    crate::webidl::define_method(scope, object, "vb", 1, vb)?;
    crate::webidl::define_method(scope, object, "vh", 1, vh)?;
    crate::webidl::define_method(scope, object, "vi", 1, vi)?;
    crate::webidl::define_method(scope, object, "vmax", 1, vmax)?;
    crate::webidl::define_method(scope, object, "vmin", 1, vmin)?;
    crate::webidl::define_method(scope, object, "vw", 1, vw)?;
    crate::webidl::define_method(scope, object, "x", 1, x)?;
    crate::webidl::define_readonly_accessor(scope, object, "paintWorklet", get_paint_worklet)?;
    let tag_key = v8::Symbol::get_to_string_tag(scope);
    let tag_value = crate::webidl::string(scope, "CSS")?;
    if object.define_own_property(
        scope,
        tag_key.into(),
        tag_value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define CSS namespace tag".to_owned());
    }
    let highlights = v8::Global::new(scope, highlights);
    let paint_worklet = v8::Global::new(scope, paint_worklet);
    let object_global = v8::Global::new(scope, object);
    let store = scope
        .get_slot_mut::<CssNamespaceStore>()
        .ok_or_else(|| "CSS namespace state was not prepared".to_owned())?;
    store.highlights = Some(highlights);
    store.paint_worklet = Some(paint_worklet);
    store.object = Some(object_global);
    crate::webidl::define_global(scope, "CSS", object.into())
}

fn get_highlights(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(highlights) = scope
        .get_slot::<CssNamespaceStore>()
        .and_then(|store| store.highlights.as_ref())
        .cloned()
    {
        result.set(v8::Local::new(scope, &highlights).into());
    }
}

fn get_paint_worklet(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(paint_worklet) = scope
        .get_slot::<CssNamespaceStore>()
        .and_then(|store| store.paint_worklet.as_ref())
        .cloned()
    {
        result.set(v8::Local::new(scope, &paint_worklet).into());
    }
}

fn unit(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    if !value.is_finite() {
        crate::webidl::throw_type_error(scope, "CSS numeric value must be finite");
        return;
    }
    match super::css_unit_value::create(scope, value, name) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn hz(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "hz");
}
fn q(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "q");
}
fn cap(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "cap");
}
fn ch(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "ch");
}
fn cm(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "cm");
}
fn cqb(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "cqb");
}
fn cqh(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "cqh");
}
fn cqi(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "cqi");
}
fn cqmax(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "cqmax");
}
fn cqmin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "cqmin");
}
fn cqw(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "cqw");
}
fn deg(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "deg");
}
fn dpcm(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "dpcm");
}
fn dpi(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "dpi");
}
fn dppx(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "dppx");
}
fn dvb(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "dvb");
}
fn dvh(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "dvh");
}
fn dvi(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "dvi");
}
fn dvmax(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "dvmax");
}
fn dvmin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "dvmin");
}
fn dvw(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "dvw");
}
fn em(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "em");
}
fn ex(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "ex");
}
fn fr(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "fr");
}
fn grad(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "grad");
}
fn ic(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "ic");
}
fn inch(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "in");
}
fn khz(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "khz");
}
fn lh(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "lh");
}
fn lvb(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "lvb");
}
fn lvh(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "lvh");
}
fn lvi(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "lvi");
}
fn lvmax(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "lvmax");
}
fn lvmin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "lvmin");
}
fn lvw(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "lvw");
}
fn mm(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "mm");
}
fn ms(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "ms");
}
fn number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "number");
}
fn pc(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "pc");
}
fn percent(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "percent");
}
fn pt(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "pt");
}
fn px(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "px");
}
fn rad(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "rad");
}
fn rcap(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "rcap");
}
fn rch(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "rch");
}
fn rem(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "rem");
}
fn rex(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "rex");
}
fn ric(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "ric");
}
fn rlh(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "rlh");
}
fn seconds(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "s");
}
fn svb(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "svb");
}
fn svh(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "svh");
}
fn svi(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "svi");
}
fn svmax(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "svmax");
}
fn svmin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "svmin");
}
fn svw(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "svw");
}
fn turn(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "turn");
}
fn vb(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "vb");
}
fn vh(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "vh");
}
fn vi(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "vi");
}
fn vmax(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "vmax");
}
fn vmin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    unit(s, a, r, "vmin");
}
fn vw(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "vw");
}
fn x(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    unit(s, a, r, "x");
}

fn escape(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let input = crate::webidl::value_to_string(scope, arguments.get(0));
    let mut output = String::new();
    for (index, character) in input.chars().enumerate() {
        if character == '\0' {
            output.push('\u{fffd}');
        } else if (index == 0 && character.is_ascii_digit())
            || (index == 1 && input.starts_with('-') && character.is_ascii_digit())
        {
            output.push('\\');
            output.push_str(&format!("{:x} ", character as u32));
        } else if character.is_ascii_alphanumeric()
            || character == '-'
            || character == '_'
            || !character.is_ascii()
        {
            output.push(character);
        } else {
            output.push('\\');
            output.push(character);
        }
    }
    if let Some(output) = v8::String::new(scope, &output) {
        result.set(output.into());
    }
}

fn supports(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let first = crate::webidl::value_to_string(scope, arguments.get(0));
    let supported = if arguments.length() > 1 {
        let second = crate::webidl::value_to_string(scope, arguments.get(1));
        super::css_style_declaration::supports_property(&first, &second)
    } else {
        first.split_once(':').is_some_and(|(name, value)| {
            super::css_style_declaration::supports_property(name, value)
        })
    };
    result.set(v8::Boolean::new(scope, supported).into());
}

fn register_property(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(definition) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Property definition is required");
        return;
    };
    let Some(name_key) = v8::String::new(scope, "name") else {
        return;
    };
    let name = definition
        .get(scope, name_key.into())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    if !name.starts_with("--") || name.len() < 3 {
        crate::webidl::throw_type_error(scope, "Custom property names must start with --");
        return;
    }
    let Some(store) = scope.get_slot_mut::<CssNamespaceStore>() else {
        return;
    };
    if !store.registered_properties.insert(name) {
        crate::webidl::throw_type_error(scope, "The custom property is already registered");
    }
}
