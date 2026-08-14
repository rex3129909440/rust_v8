#[derive(Default)]
pub(crate) struct InteractionContentfulPaintStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(InteractionContentfulPaintStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm = crate::webidl::realm_id(scope);
    let constructor = crate::webidl::create_function(
        scope,
        "InteractionContentfulPaint",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "largestContentfulPaint", get_null)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "interactionId", get_zero)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "paintTime", get_zero)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "presentationTime", get_zero)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    super::android_api_support::set_tag(scope, prototype, "InteractionContentfulPaint")?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<InteractionContentfulPaintStore>()
        .unwrap()
        .constructor
        .insert(realm, stored);
    crate::webidl::define_global(scope, "InteractionContentfulPaint", constructor.into())
}

fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn get_null(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::null(s).into());
}
fn get_zero(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::Number::new(s, 0.0).into());
}
fn to_json(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::Object::new(s).into());
}
