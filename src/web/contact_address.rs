#[derive(Default)]
pub(crate) struct ContactAddressStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ContactAddressStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm = crate::webidl::realm_id(scope);
    let constructor = crate::webidl::create_function(
        scope,
        "ContactAddress",
        0,
        v8::ConstructorBehavior::Allow,
        super::android_api_support::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::android_api_support::set_tag(scope, prototype, "ContactAddress")?;
    let stored_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ContactAddressStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    crate::webidl::define_global(scope, "ContactAddress", constructor.into())
}
