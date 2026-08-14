#[derive(Default)]
pub(crate) struct PerformanceSoftNavigationStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceSoftNavigationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm = crate::webidl::realm_id(scope);
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceSoftNavigation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "navigationType",
        get_navigation_type,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "interactionId", get_zero)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getLargestInteractionContentfulPaint",
        0,
        get_largest,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "paintTime", get_zero)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "presentationTime", get_zero)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    super::android_api_support::set_tag(scope, prototype, "PerformanceSoftNavigation")?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceSoftNavigationStore>()
        .unwrap()
        .constructor
        .insert(realm, stored);
    crate::webidl::define_global(scope, "PerformanceSoftNavigation", constructor.into())
}

fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn get_navigation_type(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = v8::String::new(s, "navigate") {
        r.set(v.into());
    }
}
fn get_zero(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::Number::new(s, 0.0).into());
}
fn get_largest(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::null(s).into());
}
