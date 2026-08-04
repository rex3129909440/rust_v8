pub(crate) struct WindowLifecycleState {
    closed: bool,
    closable: bool,
    focused: bool,
    close_attempts: u64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WindowLifecycleState {
        closed: false,
        closable: false,
        focused: true,
        close_attempts: 0,
    });
}

pub(crate) fn closed(scope: &v8::PinScope<'_, '_>) -> bool {
    scope
        .get_slot::<WindowLifecycleState>()
        .is_some_and(|state| state.closed)
}

pub(crate) fn close(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(state) = scope.get_slot_mut::<WindowLifecycleState>() {
        state.close_attempts = state.close_attempts.saturating_add(1);
        if state.closable {
            state.closed = true;
            state.focused = false;
        }
    }
}

pub(crate) fn focus(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(state) = scope.get_slot_mut::<WindowLifecycleState>() {
        if !state.closed {
            state.focused = true;
        }
    }
}

pub(crate) fn blur(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(state) = scope.get_slot_mut::<WindowLifecycleState>() {
        state.focused = false;
    }
}
