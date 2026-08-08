use super::html_input_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(scope, prototype, "files", get_files, set_files)
}

fn get_files(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        if record.input_type == "file" {
            r.set(v8::Local::new(scope, &record.files).into());
        } else {
            r.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_files(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if a.get(0).is_null() {
        if let Ok(files) = super::file_list::create(scope, Vec::new()) {
            let files = v8::Global::new(scope, files);
            update(scope, a.this(), |x| x.files = files);
        }
        return;
    }
    let Ok(files) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "files must be a FileList or null");
        return;
    };
    let is_file_list = files
        .get(scope, crate::webidl::string(scope, "item").unwrap().into())
        .is_some_and(|value| value.is_function());
    if !is_file_list {
        crate::webidl::throw_type_error(scope, "files must be a FileList or null");
        return;
    }
    let files = v8::Global::new(scope, files);
    update(scope, a.this(), |x| x.files = files);
}
