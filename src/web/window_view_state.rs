pub(crate) struct WindowViewState {
    pub(crate) inner_width: f64,
    pub(crate) inner_height: f64,
    pub(crate) scroll_x: f64,
    pub(crate) scroll_y: f64,
    pub(crate) screen_x: f64,
    pub(crate) screen_y: f64,
    pub(crate) outer_width: f64,
    pub(crate) outer_height: f64,
    pub(crate) device_pixel_ratio: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    let profile = crate::fingerprint::screen_for_isolate(isolate);
    isolate.set_slot(WindowViewState {
        inner_width: profile.viewport_width,
        inner_height: profile.viewport_height,
        scroll_x: 0.0,
        scroll_y: 0.0,
        screen_x: profile.screen_x,
        screen_y: profile.screen_y,
        outer_width: profile.outer_width,
        outer_height: profile.outer_height,
        device_pixel_ratio: profile.device_pixel_ratio,
    });
}

pub(crate) fn inner_width(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.inner_width)
        .unwrap_or(1280.0)
}

pub(crate) fn inner_height(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.inner_height)
        .unwrap_or(720.0)
}

pub(crate) fn scroll_x(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.scroll_x)
        .unwrap_or(0.0)
}

pub(crate) fn scroll_y(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.scroll_y)
        .unwrap_or(0.0)
}

pub(crate) fn screen_x(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.screen_x)
        .unwrap_or(10.0)
}

pub(crate) fn screen_y(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.screen_y)
        .unwrap_or(10.0)
}

pub(crate) fn outer_width(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.outer_width)
        .unwrap_or(1280.0)
}

pub(crate) fn outer_height(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.outer_height)
        .unwrap_or(720.0)
}

pub(crate) fn device_pixel_ratio(scope: &v8::PinScope<'_, '_>) -> f64 {
    scope
        .get_slot::<WindowViewState>()
        .map(|state| state.device_pixel_ratio)
        .unwrap_or(1.0)
}

pub(crate) fn resize_by(scope: &mut v8::PinScope<'_, '_>, width: f64, height: f64) {
    if let Some(state) = scope.get_slot_mut::<WindowViewState>() {
        state.outer_width = (state.outer_width + width).max(0.0);
        state.outer_height = (state.outer_height + height).max(0.0);
        state.inner_width = state.outer_width;
        state.inner_height = state.outer_height;
    }
}

pub(crate) fn resize_to(scope: &mut v8::PinScope<'_, '_>, width: f64, height: f64) {
    if let Some(state) = scope.get_slot_mut::<WindowViewState>() {
        state.outer_width = width.max(0.0);
        state.outer_height = height.max(0.0);
        state.inner_width = state.outer_width;
        state.inner_height = state.outer_height;
    }
}

pub(crate) fn scroll_to(scope: &mut v8::PinScope<'_, '_>, x: f64, y: f64) {
    if let Some(state) = scope.get_slot_mut::<WindowViewState>() {
        state.scroll_x = x.max(0.0);
        state.scroll_y = y.max(0.0);
    }
}

pub(crate) fn scroll_by(scope: &mut v8::PinScope<'_, '_>, x: f64, y: f64) {
    if let Some(state) = scope.get_slot_mut::<WindowViewState>() {
        state.scroll_x = (state.scroll_x + x).max(0.0);
        state.scroll_y = (state.scroll_y + y).max(0.0);
    }
}
