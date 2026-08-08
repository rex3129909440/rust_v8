pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "getDigitalGoodsService",
        1,
        v8::ConstructorBehavior::Throw,
        get_digital_goods_service,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "getDigitalGoodsService")?;
    match global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) {
        Some(true) => Ok(()),
        _ => Err("cannot define window.getDigitalGoodsService".to_owned()),
    }
}

fn get_digital_goods_service(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'getDigitalGoodsService' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let provider = crate::webidl::value_to_string(scope, arguments.get(0));
    let service = match create_service(scope, &provider) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, service.into()) {
        result.set(promise.into());
    }
}

fn create_service<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    provider: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let prototype = v8::Object::new(scope);
    crate::webidl::define_method(scope, prototype, "getDetails", 1, get_details)?;
    crate::webidl::define_method(scope, prototype, "listPurchases", 0, list_purchases)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "listPurchaseHistory",
        0,
        list_purchase_history,
    )?;
    crate::webidl::define_method(scope, prototype, "consume", 1, consume)?;
    let service = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, service, prototype.into()) != Some(true) {
        return Err("cannot create DigitalGoodsService".to_owned());
    }
    let provider_key = crate::webidl::string(scope, "paymentMethod")?;
    let provider_value = crate::webidl::string(scope, provider)?;
    let _ = service.define_own_property(
        scope,
        provider_key.into(),
        provider_value.into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_ENUM,
    );
    Ok(service)
}

fn get_details(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let values = v8::Local::<v8::Array>::try_from(arguments.get(0));
    let Ok(values) = values else {
        crate::webidl::throw_type_error(scope, "Item identifiers must be an array.");
        return;
    };
    let details = v8::Array::new(scope, values.length() as i32);
    for index in 0..values.length() {
        let identifier = values
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let item = v8::Object::new(scope);
        let item_id = v8::String::new(scope, "itemId").expect("short key");
        let title = v8::String::new(scope, "title").expect("short key");
        let description = v8::String::new(scope, "description").expect("short key");
        let price = v8::String::new(scope, "price").expect("short key");
        let item_type = v8::String::new(scope, "type").expect("short key");
        let title_value = crate::webidl::value_to_string(scope, identifier);
        let title_value = v8::String::new(scope, &title_value).expect("valid item title");
        let description_value =
            v8::String::new(scope, "Offline catalog item").expect("short value");
        let price_value = v8::Object::new(scope);
        let currency_key = v8::String::new(scope, "currency").expect("short key");
        let value_key = v8::String::new(scope, "value").expect("short key");
        let currency = v8::String::new(scope, "USD").expect("short value");
        let zero = v8::String::new(scope, "0.00").expect("short value");
        let _ = price_value.set(scope, currency_key.into(), currency.into());
        let _ = price_value.set(scope, value_key.into(), zero.into());
        let product = v8::String::new(scope, "product").expect("short value");
        let _ = item.set(scope, item_id.into(), identifier);
        let _ = item.set(scope, title.into(), title_value.into());
        let _ = item.set(scope, description.into(), description_value.into());
        let _ = item.set(scope, price.into(), price_value.into());
        let _ = item.set(scope, item_type.into(), product.into());
        let _ = details.set_index(scope, index, item.into());
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, details.into()) {
        result.set(promise.into());
    }
}

fn list_purchases(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let purchases = v8::Array::new(scope, 0);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, purchases.into()) {
        result.set(promise.into());
    }
}

fn list_purchase_history(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let history = v8::Array::new(scope, 0);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, history.into()) {
        result.set(promise.into());
    }
}

fn consume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "A purchase token is required.");
        return;
    }
    let undefined = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
        result.set(promise.into());
    }
}
