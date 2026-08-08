use super::html_i_frame_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "adAuctionHeaders",
        get_ad_auction_headers,
        set_ad_auction_headers,
    )
}

fn get_ad_auction_headers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_flag(scope, arguments, result, |record| record.ad_auction_headers);
}

fn set_ad_auction_headers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.ad_auction_headers = value
    });
}
