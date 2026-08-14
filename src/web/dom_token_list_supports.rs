pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "supports", 1, supports)
}
fn supports(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(support) = super::dom_token_list::support(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'supports' on 'DOMTokenList': 1 argument required, but only 0 present.",
        );
        return;
    }
    let value = arguments.get(0);
    if value.is_symbol() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'supports' on 'DOMTokenList': Cannot convert a Symbol value to a string",
        );
        return;
    }
    let Some(token) = crate::webidl::dom_string(scope, value) else {
        return;
    };
    use super::dom_token_list::DomTokenSupport;
    let token = token.to_ascii_lowercase();
    let supported = match support {
        DomTokenSupport::None => {
            crate::webidl::throw_type_error(
                scope,
                "Failed to execute 'supports' on 'DOMTokenList': DOMTokenList has no supported tokens.",
            );
            return;
        }
        DomTokenSupport::HyperlinkRel => {
            matches!(token.as_str(), "noopener" | "noreferrer" | "opener")
        }
        DomTokenSupport::LinkRel => matches!(
            token.as_str(),
            "alternate"
                | "apple-touch-icon"
                | "canonical"
                | "compression-dictionary"
                | "dns-prefetch"
                | "icon"
                | "manifest"
                | "modulepreload"
                | "next"
                | "preconnect"
                | "prefetch"
                | "preload"
                | "prerender"
                | "stylesheet"
        ),
        DomTokenSupport::Sandbox => matches!(
            token.as_str(),
            "allow-downloads"
                | "allow-forms"
                | "allow-modals"
                | "allow-orientation-lock"
                | "allow-pointer-lock"
                | "allow-popups"
                | "allow-popups-to-escape-sandbox"
                | "allow-presentation"
                | "allow-same-origin"
                | "allow-scripts"
                | "allow-storage-access-by-user-activation"
                | "allow-top-navigation"
                | "allow-top-navigation-by-user-activation"
        ),
        DomTokenSupport::Blocking => token == "render",
        DomTokenSupport::MediaControls => {
            matches!(
                token.as_str(),
                "nodownload" | "nofullscreen" | "noremoteplayback"
            )
        }
    };
    result.set(v8::Boolean::new(scope, supported).into())
}
