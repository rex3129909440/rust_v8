pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "scrollIntoView", 0, scroll_into_view)
}

fn scroll_into_view(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if super::element::record(scope, arguments.this()).is_none() {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "Element",
            "scrollIntoView",
            result,
        );
        return;
    }
    if !super::element_method_support::ensure(scope, arguments.this()) {
        return;
    }
    let Some(options) = options(scope, &arguments) else {
        return;
    };
    scroll_element_into_view(
        scope,
        arguments.this(),
        options.block,
        options.inline,
        false,
        options.nearest_container,
    );
    if let Ok(promise) = super::element_method_support::resolved_undefined(scope) {
        result.set(promise.into());
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Alignment {
    Start,
    Center,
    End,
    Nearest,
}

struct ScrollIntoViewOptions {
    block: Alignment,
    inline: Alignment,
    nearest_container: bool,
}

pub(crate) fn scroll_element_into_view(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    block: Alignment,
    inline: Alignment,
    center_only_if_needed: bool,
    nearest_container: bool,
) {
    let mut parent = composed_parent(scope, target);
    while let Some(candidate) = parent {
        let next = composed_parent(scope, candidate);
        if super::element::record(scope, candidate).is_none() {
            parent = next;
            continue;
        }
        let metrics = super::element_layout::scroll_metrics(scope, candidate);
        let scrollable = metrics.scroll_width > metrics.client_width
            || metrics.scroll_height > metrics.client_height;
        if !scrollable {
            parent = next;
            continue;
        }
        let ancestor_layout = super::element_layout::compute(scope, candidate);
        let target_rect = super::element_layout::compute(scope, target).rect();
        let Some(record) = super::element::record(scope, candidate) else {
            parent = next;
            continue;
        };
        let origin_x = ancestor_layout.x + ancestor_layout.border_left;
        let origin_y = ancestor_layout.y + ancestor_layout.border_top;
        let target_left = target_rect.x - origin_x + record.scroll_left;
        let target_top = target_rect.y - origin_y + record.scroll_top;
        let target_right = target_left + target_rect.width;
        let target_bottom = target_top + target_rect.height;
        let fully_visible = target_left >= record.scroll_left
            && target_right <= record.scroll_left + metrics.client_width
            && target_top >= record.scroll_top
            && target_bottom <= record.scroll_top + metrics.client_height;
        if !(center_only_if_needed && fully_visible) {
            let next_left = aligned_offset(
                inline,
                target_left,
                target_right,
                record.scroll_left,
                metrics.client_width,
            );
            let next_top = aligned_offset(
                block,
                target_top,
                target_bottom,
                record.scroll_top,
                metrics.client_height,
            );
            let _ = super::element::set_scroll_position(
                scope,
                candidate,
                next_left.round(),
                next_top.round(),
                false,
            );
        }
        if nearest_container {
            break;
        }
        parent = next;
    }
}

fn aligned_offset(
    alignment: Alignment,
    start: f64,
    end: f64,
    current: f64,
    viewport_size: f64,
) -> f64 {
    match alignment {
        Alignment::Start => start,
        Alignment::Center => (start + end - viewport_size) / 2.0,
        Alignment::End => end - viewport_size,
        Alignment::Nearest => {
            if start < current {
                start
            } else if end > current + viewport_size {
                end - viewport_size
            } else {
                current
            }
        }
    }
}

fn options(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Option<ScrollIntoViewOptions> {
    let value = arguments.get(0);
    if value.is_undefined() {
        return Some(ScrollIntoViewOptions {
            block: Alignment::Start,
            inline: Alignment::Nearest,
            nearest_container: false,
        });
    }
    if value.is_boolean() {
        return Some(ScrollIntoViewOptions {
            block: if value.boolean_value(scope) {
                Alignment::Start
            } else {
                Alignment::End
            },
            inline: Alignment::Nearest,
            nearest_container: false,
        });
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(
            scope,
            "The provided value is not a valid ScrollIntoViewOptions",
        );
        return None;
    };
    if !valid_behavior(scope, object) {
        return None;
    }
    Some(ScrollIntoViewOptions {
        block: alignment_member(scope, object, "block", Alignment::Start)?,
        inline: alignment_member(scope, object, "inline", Alignment::Nearest)?,
        nearest_container: container_member(scope, object)?,
    })
}

fn alignment_member(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    fallback: Alignment,
) -> Option<Alignment> {
    let Some(value) = member(scope, object, name) else {
        return Some(fallback);
    };
    match crate::webidl::value_to_string(scope, value).as_str() {
        "start" => Some(Alignment::Start),
        "center" => Some(Alignment::Center),
        "end" => Some(Alignment::End),
        "nearest" => Some(Alignment::Nearest),
        value => {
            crate::webidl::throw_type_error(
                scope,
                &format!("'{value}' is not a valid ScrollLogicalPosition"),
            );
            None
        }
    }
}

fn valid_behavior(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    let Some(value) = member(scope, object, "behavior") else {
        return true;
    };
    let value = crate::webidl::value_to_string(scope, value);
    if matches!(value.as_str(), "auto" | "instant" | "smooth") {
        true
    } else {
        crate::webidl::throw_type_error(scope, &format!("'{value}' is not a valid ScrollBehavior"));
        false
    }
}

fn container_member(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<bool> {
    let Some(value) = member(scope, object, "container") else {
        return Some(false);
    };
    match crate::webidl::value_to_string(scope, value).as_str() {
        "all" => Some(false),
        "nearest" => Some(true),
        value => {
            crate::webidl::throw_type_error(
                scope,
                &format!("'{value}' is not a valid ScrollIntoViewContainer"),
            );
            None
        }
    }
}

fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    object
        .get(scope, v8::String::new(scope, name)?.into())
        .filter(|value| !value.is_undefined())
}

fn composed_parent<'s>(
    scope: &v8::PinScope<'s, '_>,
    node: v8::Local<'s, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::node::parent(scope, node).or_else(|| super::shadow_root::host(scope, node))
}
