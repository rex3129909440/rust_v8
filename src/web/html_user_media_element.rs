#[derive(Default)]
pub(crate) struct HtmlUserMediaElementStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlUserMediaElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm = crate::webidl::realm_id(scope);
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLUserMediaElement",
        0,
        v8::ConstructorBehavior::Allow,
        super::html_body_element::illegal_constructor,
    )?;
    let parent = super::html_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "error", get_null)?;
    crate::webidl::define_accessor(scope, prototype, "onstream", get_null, set_noop)?;
    crate::webidl::define_accessor(scope, prototype, "oncancel", get_null, set_noop)?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_null, set_noop)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "stream", get_null)?;
    crate::webidl::define_method(scope, prototype, "setConstraints", 0, set_constraints)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::android_api_support::set_tag(scope, prototype, "HTMLUserMediaElement")?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlUserMediaElementStore>()
        .unwrap()
        .constructor
        .insert(realm, stored);
    crate::webidl::define_global(scope, "HTMLUserMediaElement", constructor.into())
}

fn get_null(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::null(s).into());
}
fn set_noop(
    _: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
}
fn set_constraints(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(p) = super::android_api_support::resolved_undefined(s) {
        r.set(p.into());
    }
}
