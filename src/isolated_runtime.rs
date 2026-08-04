use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const WORKER_ARGUMENT: &str = "__edge_sandbox_worker";
const WORKER_ENVIRONMENT: &str = "EDGE_SANDBOX_ISOLATED_WORKER";
const MAX_IPC_FRAME_BYTES: usize = 64 * 1024 * 1024;
const STDERR_CAPTURE_BYTES: usize = 64 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(5);
const EVALUATION_GRACE: Duration = Duration::from_millis(100);
const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const CONTROL_TIMEOUT: Duration = Duration::from_secs(5);
const TRACE_EXPORT_BATCH_SIZE: u64 = 8_192;

#[derive(Debug, Deserialize, Serialize)]
enum WorkerCommand {
    Initialize(Box<crate::EdgeRuntimeOptions>),
    Evaluate {
        source: String,
        source_url: Option<String>,
    },
    EnableProxyTrace,
    DisableProxyTrace,
    ClearProxyTrace,
    SetNativeTraceExclusions(Vec<String>),
    ProxyTrace,
    ProxyTraceMatching(String),
    NetworkRequests,
    ClearNetworkRequests,
    Stdout,
    ClearStdout,
    Shutdown,
}

#[derive(Debug, Deserialize, Serialize)]
enum WorkerResponse {
    Initialized(Result<(), String>),
    Evaluation(Result<crate::Evaluation, String>),
    Unit(Result<(), String>),
    Trace(Result<Vec<crate::TraceEntry>, String>),
    NetworkRequests(Result<Vec<crate::CapturedNetworkRequest>, String>),
    Stdout(Result<Vec<crate::CapturedConsoleOutput>, String>),
}

struct ControllerState {
    process: Option<WorkerProcess>,
    trace_enabled: bool,
    trace_exclusions: Vec<String>,
}

/// An Edge runtime hosted in a dedicated operating-system process.
///
/// All JavaScript and V8 state lives in the worker. The controller only owns
/// bounded binary IPC pipes and the process handle, so a fatal V8 failure,
/// resident-memory breach, or unresponsive execution can be terminated
/// without taking down the embedding process.
pub struct IsolatedEdgeRuntime {
    launcher: WorkerLauncher,
    options: crate::EdgeRuntimeOptions,
    state: Mutex<ControllerState>,
}

#[derive(Clone)]
enum WorkerLauncher {
    Executable(PathBuf),
    SharedLibrary,
}

impl IsolatedEdgeRuntime {
    /// Starts a worker by re-executing the current program.
    ///
    /// Applications embedding the library should either route
    /// [`run_isolated_worker`] from their hidden worker argument or use
    /// [`Self::with_worker_executable`] with the `edge-sandbox` executable.
    pub fn new(options: crate::EdgeRuntimeOptions) -> Result<Self, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("cannot locate isolated worker executable: {error}"))?;
        Self::with_worker_executable(options, executable)
    }

    /// Starts a process-isolated worker from the already-loaded dynamic
    /// library. No edge-sandbox worker executable is required at deployment.
    pub fn self_hosted(mut options: crate::EdgeRuntimeOptions) -> Result<Self, String> {
        options.limits.apply_isolated_defaults();
        options.validate()?;
        let launcher = WorkerLauncher::SharedLibrary;
        let process = WorkerProcess::spawn(&launcher, options.clone())?;
        Ok(Self {
            launcher,
            options,
            state: Mutex::new(ControllerState {
                process: Some(process),
                trace_enabled: false,
                trace_exclusions: Vec::new(),
            }),
        })
    }

    /// Starts a worker using an executable that implements the hidden worker
    /// entry point. This is the appropriate constructor for library embedders.
    pub fn with_worker_executable(
        mut options: crate::EdgeRuntimeOptions,
        executable: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        options.limits.apply_isolated_defaults();
        options.validate()?;
        let executable = executable.into();
        if !executable.is_file() {
            return Err(format!(
                "isolated worker executable does not exist: {}",
                executable.display()
            ));
        }
        let launcher = WorkerLauncher::Executable(executable);
        let process = WorkerProcess::spawn(&launcher, options.clone())?;
        Ok(Self {
            launcher,
            options,
            state: Mutex::new(ControllerState {
                process: Some(process),
                trace_enabled: false,
                trace_exclusions: Vec::new(),
            }),
        })
    }

    pub fn evaluate(&self, source: &str) -> Result<crate::Evaluation, String> {
        self.evaluate_internal(source, None)
    }

    pub fn evaluate_with_source_url(
        &self,
        source: &str,
        source_url: &str,
    ) -> Result<crate::Evaluation, String> {
        if source_url.is_empty() {
            return Err("JavaScript source URL cannot be empty".to_owned());
        }
        self.evaluate_internal(source, Some(source_url))
    }

    fn evaluate_internal(
        &self,
        source: &str,
        source_url: Option<&str>,
    ) -> Result<crate::Evaluation, String> {
        if self
            .options
            .limits
            .max_source_bytes
            .is_some_and(|maximum| source.len() > maximum)
        {
            return Err("JavaScript source exceeded max_source_bytes".to_owned());
        }
        let deadline = self
            .options
            .limits
            .timeout
            .unwrap_or(Duration::from_secs(30))
            .saturating_add(EVALUATION_GRACE);
        match self.request(
            WorkerCommand::Evaluate {
                source: source.to_owned(),
                source_url: source_url.map(str::to_owned),
            },
            deadline,
        )? {
            WorkerResponse::Evaluation(result) => result,
            _ => Err(self.protocol_failure("evaluation")),
        }
    }

    pub fn enable_native_trace(&self) -> Result<(), String> {
        let mut state = self.lock_state()?;
        let response =
            self.request_locked(&mut state, WorkerCommand::EnableProxyTrace, CONTROL_TIMEOUT);
        match response {
            Ok(WorkerResponse::Unit(result)) => {
                result?;
                state.trace_enabled = true;
                Ok(())
            }
            Ok(_) => Err("isolated worker returned an invalid trace-enable response".to_owned()),
            Err(message) => Err(message),
        }
    }

    pub fn enable_proxy_trace(&self) -> Result<(), String> {
        self.enable_native_trace()
    }

    pub fn disable_native_trace(&self) -> Result<(), String> {
        let mut state = self.lock_state()?;
        state.trace_enabled = false;
        match self.request_locked(
            &mut state,
            WorkerCommand::DisableProxyTrace,
            CONTROL_TIMEOUT,
        )? {
            WorkerResponse::Unit(result) => result,
            _ => Err("isolated worker returned an invalid trace-disable response".to_owned()),
        }
    }

    pub fn disable_proxy_trace(&self) -> Result<(), String> {
        self.disable_native_trace()
    }

    pub fn clear_native_trace(&self) -> Result<(), String> {
        match self.request(WorkerCommand::ClearProxyTrace, CONTROL_TIMEOUT)? {
            WorkerResponse::Unit(result) => result,
            _ => Err("isolated worker returned an invalid trace-clear response".to_owned()),
        }
    }

    pub fn clear_proxy_trace(&self) -> Result<(), String> {
        self.clear_native_trace()
    }

    pub fn set_native_trace_exclusions(&self, exclusions: &[String]) -> Result<(), String> {
        let mut state = self.lock_state()?;
        let exclusions = exclusions.to_vec();
        match self.request_locked(
            &mut state,
            WorkerCommand::SetNativeTraceExclusions(exclusions.clone()),
            CONTROL_TIMEOUT,
        )? {
            WorkerResponse::Unit(result) => {
                result?;
                state.trace_exclusions = exclusions;
                Ok(())
            }
            _ => Err("isolated worker returned an invalid trace-exclusion response".to_owned()),
        }
    }

    pub fn native_trace(&self) -> Result<Vec<crate::TraceEntry>, String> {
        let mut entries = Vec::new();
        let mut start = 1_u64;
        loop {
            let end = start.saturating_add(TRACE_EXPORT_BATCH_SIZE - 1);
            let batch = self.native_trace_matching(&format!("@sequence:{start}..{end}"))?;
            let batch_len = batch.len();
            entries.extend(batch);
            if batch_len < TRACE_EXPORT_BATCH_SIZE as usize || end == u64::MAX {
                break;
            }
            start = end + 1;
        }
        Ok(entries)
    }

    pub fn proxy_trace(&self) -> Result<Vec<crate::TraceEntry>, String> {
        self.native_trace()
    }

    pub fn native_trace_matching(&self, needle: &str) -> Result<Vec<crate::TraceEntry>, String> {
        match self.request(
            WorkerCommand::ProxyTraceMatching(needle.to_owned()),
            CONTROL_TIMEOUT,
        )? {
            WorkerResponse::Trace(result) => result,
            _ => Err("isolated worker returned an invalid filtered trace response".to_owned()),
        }
    }

    pub fn proxy_trace_matching(&self, needle: &str) -> Result<Vec<crate::TraceEntry>, String> {
        self.native_trace_matching(needle)
    }

    pub fn network_requests(&self) -> Result<Vec<crate::CapturedNetworkRequest>, String> {
        match self.request(WorkerCommand::NetworkRequests, CONTROL_TIMEOUT)? {
            WorkerResponse::NetworkRequests(result) => result,
            _ => Err("isolated worker returned an invalid network-request response".to_owned()),
        }
    }

    pub fn clear_network_requests(&self) -> Result<(), String> {
        match self.request(WorkerCommand::ClearNetworkRequests, CONTROL_TIMEOUT)? {
            WorkerResponse::Unit(result) => result,
            _ => {
                Err("isolated worker returned an invalid network-request-clear response".to_owned())
            }
        }
    }

    pub fn stdout(&self) -> Result<Vec<crate::CapturedConsoleOutput>, String> {
        match self.request(WorkerCommand::Stdout, CONTROL_TIMEOUT)? {
            WorkerResponse::Stdout(result) => result,
            _ => Err("isolated worker returned an invalid stdout response".to_owned()),
        }
    }

    pub fn clear_stdout(&self) -> Result<(), String> {
        match self.request(WorkerCommand::ClearStdout, CONTROL_TIMEOUT)? {
            WorkerResponse::Unit(result) => result,
            _ => Err("isolated worker returned an invalid stdout-clear response".to_owned()),
        }
    }

    /// Returns the worker PID, which is always different from the controller
    /// PID after successful construction.
    pub fn process_id(&self) -> Result<u32, String> {
        let mut state = self.lock_state()?;
        self.ensure_process(&mut state)?;
        state
            .process
            .as_ref()
            .map(|process| process.child.id())
            .ok_or_else(|| "isolated worker is unavailable".to_owned())
    }

    /// Samples the worker's resident set size when supported by the host OS.
    pub fn resident_memory_bytes(&self) -> Result<Option<u64>, String> {
        let mut state = self.lock_state()?;
        self.ensure_process(&mut state)?;
        Ok(state
            .process
            .as_ref()
            .and_then(|process| process_resident_memory_bytes(&process.child)))
    }

    fn request(
        &self,
        command: WorkerCommand,
        deadline: Duration,
    ) -> Result<WorkerResponse, String> {
        let mut state = self.lock_state()?;
        self.request_locked(&mut state, command, deadline)
    }

    fn request_locked(
        &self,
        state: &mut ControllerState,
        command: WorkerCommand,
        deadline: Duration,
    ) -> Result<WorkerResponse, String> {
        self.ensure_process(state)?;
        let result = state.process.as_mut().expect("worker was ensured").request(
            command,
            deadline,
            self.options.limits.max_resident_bytes,
        );
        if result.is_err()
            && let Some(mut process) = state.process.take()
        {
            process.terminate();
        }
        result
    }

    fn ensure_process(&self, state: &mut ControllerState) -> Result<(), String> {
        if state.process.is_some() {
            return Ok(());
        }
        let mut process = WorkerProcess::spawn(&self.launcher, self.options.clone())?;
        if !state.trace_exclusions.is_empty() {
            match process.request(
                WorkerCommand::SetNativeTraceExclusions(state.trace_exclusions.clone()),
                CONTROL_TIMEOUT,
                self.options.limits.max_resident_bytes,
            ) {
                Ok(WorkerResponse::Unit(Ok(()))) => {}
                Ok(WorkerResponse::Unit(Err(message))) => {
                    process.terminate();
                    return Err(message);
                }
                Ok(_) => {
                    process.terminate();
                    return Err(
                        "isolated worker returned an invalid trace-exclusion response".to_owned(),
                    );
                }
                Err(message) => {
                    process.terminate();
                    return Err(message);
                }
            }
        }
        if state.trace_enabled {
            match process.request(
                WorkerCommand::EnableProxyTrace,
                CONTROL_TIMEOUT,
                self.options.limits.max_resident_bytes,
            ) {
                Ok(WorkerResponse::Unit(Ok(()))) => {}
                Ok(WorkerResponse::Unit(Err(message))) => {
                    process.terminate();
                    return Err(message);
                }
                Ok(_) => {
                    process.terminate();
                    return Err(
                        "isolated worker returned an invalid trace-enable response".to_owned()
                    );
                }
                Err(message) => {
                    process.terminate();
                    return Err(message);
                }
            }
        }
        state.process = Some(process);
        Ok(())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, ControllerState>, String> {
        self.state
            .lock()
            .map_err(|_| "isolated worker controller lock was poisoned".to_owned())
    }

    fn protocol_failure(&self, operation: &str) -> String {
        format!("isolated worker returned an invalid {operation} response")
    }
}

impl Drop for IsolatedEdgeRuntime {
    fn drop(&mut self) {
        let Ok(state) = self.state.get_mut() else {
            return;
        };
        let Some(mut process) = state.process.take() else {
            return;
        };
        let _ = process.request(WorkerCommand::Shutdown, CONTROL_TIMEOUT, None);
        process.terminate();
    }
}

enum WorkerChild {
    Spawned(Child),
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    Forked {
        pid: libc::pid_t,
        status: Option<ExitStatus>,
    },
}

impl WorkerChild {
    fn id(&self) -> u32 {
        match self {
            Self::Spawned(child) => child.id(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            Self::Forked { pid, .. } => *pid as u32,
        }
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        match self {
            Self::Spawned(child) => child.try_wait(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            Self::Forked { pid, status } => {
                use std::os::unix::process::ExitStatusExt;

                if let Some(status) = status {
                    return Ok(Some(*status));
                }
                let mut raw_status = 0;
                // SAFETY: `pid` names the uniquely owned worker child and the
                // status pointer is valid for this non-blocking wait.
                let waited = unsafe { libc::waitpid(*pid, &mut raw_status, libc::WNOHANG) };
                if waited == 0 {
                    Ok(None)
                } else if waited == *pid {
                    let value = ExitStatus::from_raw(raw_status);
                    *status = Some(value);
                    Ok(Some(value))
                } else {
                    Err(std::io::Error::last_os_error())
                }
            }
        }
    }

    fn kill(&mut self) -> std::io::Result<()> {
        match self {
            Self::Spawned(child) => child.kill(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            Self::Forked { pid, status } => {
                if status.is_some() {
                    return Ok(());
                }
                // SAFETY: sending SIGKILL to the recorded child PID cannot
                // affect the controller process.
                let result = unsafe { libc::kill(*pid, libc::SIGKILL) };
                if result == 0 {
                    Ok(())
                } else {
                    let error = std::io::Error::last_os_error();
                    if error.raw_os_error() == Some(libc::ESRCH) {
                        Ok(())
                    } else {
                        Err(error)
                    }
                }
            }
        }
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        match self {
            Self::Spawned(child) => child.wait(),
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            Self::Forked { pid, status } => {
                use std::os::unix::process::ExitStatusExt;

                if let Some(status) = status {
                    return Ok(*status);
                }
                loop {
                    let mut raw_status = 0;
                    // SAFETY: `pid` names the uniquely owned worker child and
                    // this blocking wait is performed only during teardown.
                    let waited = unsafe { libc::waitpid(*pid, &mut raw_status, 0) };
                    if waited == *pid {
                        let value = ExitStatus::from_raw(raw_status);
                        *status = Some(value);
                        return Ok(value);
                    }
                    let error = std::io::Error::last_os_error();
                    if error.kind() != std::io::ErrorKind::Interrupted {
                        return Err(error);
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    fn spawned(&self) -> &Child {
        let Self::Spawned(child) = self;
        child
    }
}

struct SpawnedWorker {
    child: WorkerChild,
    stdin: Box<dyn Write + Send>,
    stdout: Box<dyn Read + Send>,
    stderr: Box<dyn Read + Send>,
}

fn spawn_worker(
    launcher: &WorkerLauncher,
    options: &crate::EdgeRuntimeOptions,
) -> Result<SpawnedWorker, String> {
    match launcher {
        WorkerLauncher::Executable(executable) => {
            let mut command = Command::new(executable);
            command.arg(WORKER_ARGUMENT);
            spawn_worker_command(command, options)
        }
        WorkerLauncher::SharedLibrary => spawn_shared_library_worker(options),
    }
}

fn spawn_worker_command(
    mut command: Command,
    options: &crate::EdgeRuntimeOptions,
) -> Result<SpawnedWorker, String> {
    command
        .env_clear()
        .env(WORKER_ENVIRONMENT, "1")
        .env("TZ", &options.fingerprint.locale.time_zone)
        .env("LC_ALL", "C")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    preserve_runtime_environment(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("cannot start isolated Edge worker: {error}"))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "isolated worker stdin pipe was not created".to_owned())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "isolated worker stdout pipe was not created".to_owned())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "isolated worker stderr pipe was not created".to_owned())?;
    Ok(SpawnedWorker {
        child: WorkerChild::Spawned(child),
        stdin: Box::new(stdin),
        stdout: Box::new(stdout),
        stderr: Box::new(stderr),
    })
}

#[cfg(windows)]
fn spawn_shared_library_worker(
    options: &crate::EdgeRuntimeOptions,
) -> Result<SpawnedWorker, String> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let library = current_dynamic_library_path()?;
    let system_root = std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .ok_or_else(|| "cannot locate the Windows system directory".to_owned())?;
    let loader = PathBuf::from(system_root)
        .join("System32")
        .join("rundll32.exe");
    if !loader.is_file() {
        return Err(format!(
            "Windows system DLL loader does not exist: {}",
            loader.display()
        ));
    }
    let mut command = Command::new(loader);
    command
        .raw_arg(format!(
            "\"{}\",edge_sandbox_worker_entry",
            library.display()
        ))
        .creation_flags(CREATE_NO_WINDOW);
    spawn_worker_command(command, options)
}

#[cfg(windows)]
fn current_dynamic_library_path() -> Result<PathBuf, String> {
    const GET_MODULE_HANDLE_EX_FLAG_PIN: u32 = 0x0000_0001;
    const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x0000_0004;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleExW(
            flags: u32,
            module_name: *const u16,
            module: *mut *mut std::ffi::c_void,
        ) -> i32;
        fn GetModuleFileNameW(module: *mut std::ffi::c_void, filename: *mut u16, size: u32) -> u32;
    }

    let mut module = std::ptr::null_mut();
    // SAFETY: with FROM_ADDRESS, module_name is interpreted as an address
    // inside the desired module instead of as a UTF-16 string.
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            edge_sandbox_worker_entry as *const () as *const u16,
            &mut module,
        )
    };
    if found == 0 {
        return Err(format!(
            "cannot locate the loaded edge-sandbox DLL: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut capacity = 512_usize;
    loop {
        let mut buffer = vec![0_u16; capacity];
        // SAFETY: module is pinned above and buffer has exactly `capacity`
        // writable UTF-16 code units.
        let length = unsafe { GetModuleFileNameW(module, buffer.as_mut_ptr(), capacity as u32) };
        if length == 0 {
            return Err(format!(
                "cannot resolve the loaded edge-sandbox DLL path: {}",
                std::io::Error::last_os_error()
            ));
        }
        if (length as usize) < capacity - 1 {
            buffer.truncate(length as usize);
            return Ok(PathBuf::from(String::from_utf16_lossy(&buffer)));
        }
        capacity = capacity
            .checked_mul(2)
            .filter(|value| *value <= 32_768)
            .ok_or_else(|| "loaded edge-sandbox DLL path is too long".to_owned())?;
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn spawn_shared_library_worker(
    _options: &crate::EdgeRuntimeOptions,
) -> Result<SpawnedWorker, String> {
    use std::fs::File;
    use std::os::fd::FromRawFd;

    fn pipe() -> Result<[libc::c_int; 2], String> {
        let mut descriptors = [-1; 2];
        // SAFETY: the two-element array is writable and libc initializes both
        // descriptors on success.
        if unsafe { libc::pipe(descriptors.as_mut_ptr()) } != 0 {
            return Err(format!(
                "cannot create isolated worker pipe: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(descriptors)
    }

    let request = pipe()?;
    let response = match pipe() {
        Ok(pipe) => pipe,
        Err(message) => {
            // SAFETY: both descriptors were returned by the successful pipe.
            unsafe {
                libc::close(request[0]);
                libc::close(request[1]);
            }
            return Err(message);
        }
    };
    let errors = match pipe() {
        Ok(pipe) => pipe,
        Err(message) => {
            // SAFETY: all four descriptors were returned by successful pipes.
            unsafe {
                libc::close(request[0]);
                libc::close(request[1]);
                libc::close(response[0]);
                libc::close(response[1]);
            }
            return Err(message);
        }
    };

    // SAFETY: the process is intentionally forked before any EdgeRuntime/V8
    // state is constructed. The child communicates only through these pipes.
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        let error = std::io::Error::last_os_error();
        // SAFETY: each descriptor is live and uniquely owned here.
        unsafe {
            for descriptor in [
                request[0],
                request[1],
                response[0],
                response[1],
                errors[0],
                errors[1],
            ] {
                libc::close(descriptor);
            }
        }
        return Err(format!("cannot fork isolated Edge worker: {error}"));
    }
    if pid == 0 {
        // SAFETY: this branch runs only in the new child. It closes the parent
        // pipe ends and redirects diagnostics to the controller's error pipe.
        unsafe {
            libc::close(request[1]);
            libc::close(response[0]);
            libc::close(errors[0]);
            libc::dup2(errors[1], libc::STDERR_FILENO);
            libc::close(errors[1]);
        }
        apply_parent_death_policy();
        apply_network_policy();
        // SAFETY: the child uniquely owns these two remaining descriptors.
        let input = unsafe { File::from_raw_fd(request[0]) };
        let output = unsafe { File::from_raw_fd(response[1]) };
        let status = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            run_isolated_worker_protocol(input, output)
        })) {
            Ok(Ok(())) => 0,
            Ok(Err(message)) => {
                eprintln!("{message}");
                70
            }
            Err(_) => {
                eprintln!("isolated Edge worker panicked");
                71
            }
        };
        // SAFETY: bypassing parent-process destructors is required after fork.
        unsafe { libc::_exit(status) }
    }

    // SAFETY: the controller closes the child pipe ends and assumes ownership
    // of the remaining descriptors exactly once below.
    unsafe {
        libc::close(request[0]);
        libc::close(response[1]);
        libc::close(errors[1]);
    }
    Ok(SpawnedWorker {
        child: WorkerChild::Forked { pid, status: None },
        // SAFETY: each descriptor is live and uniquely owned by its File.
        stdin: Box::new(unsafe { File::from_raw_fd(request[1]) }),
        stdout: Box::new(unsafe { File::from_raw_fd(response[0]) }),
        stderr: Box::new(unsafe { File::from_raw_fd(errors[0]) }),
    })
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn spawn_shared_library_worker(
    _options: &crate::EdgeRuntimeOptions,
) -> Result<SpawnedWorker, String> {
    Err("self-hosted dynamic-library workers are unsupported on this platform".to_owned())
}

struct WorkerProcess {
    child: WorkerChild,
    stdin: Box<dyn Write + Send>,
    responses: mpsc::Receiver<Result<WorkerResponse, String>>,
    stdout_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<()>>,
    stderr: Arc<Mutex<LimitedBuffer>>,
    #[cfg(windows)]
    job: Option<WindowsJob>,
}

impl WorkerProcess {
    fn spawn(
        launcher: &WorkerLauncher,
        options: crate::EdgeRuntimeOptions,
    ) -> Result<Self, String> {
        let SpawnedWorker {
            child,
            stdin,
            mut stdout,
            stderr,
        } = spawn_worker(launcher, &options)?;

        #[cfg(windows)]
        let job = match WindowsJob::attach(&child, options.limits.max_resident_bytes) {
            Ok(job) => Some(job),
            Err(message) => {
                let mut child = child;
                let _ = child.kill();
                let _ = child.wait();
                return Err(message);
            }
        };
        let (response_sender, responses) = mpsc::channel();
        let stdout_thread = thread::Builder::new()
            .name("edge-sandbox-worker-stdout".to_owned())
            .spawn(move || {
                loop {
                    let response =
                        read_frame::<_, WorkerResponse>(&mut stdout, MAX_IPC_FRAME_BYTES)
                            .map_err(|error| error.to_string());
                    let failed = response.is_err();
                    if response_sender.send(response).is_err() || failed {
                        break;
                    }
                }
            })
            .map_err(|error| format!("cannot start isolated worker response reader: {error}"))?;
        let captured_stderr = Arc::new(Mutex::new(LimitedBuffer::default()));
        let stderr_target = Arc::clone(&captured_stderr);
        let stderr_thread = thread::Builder::new()
            .name("edge-sandbox-worker-stderr".to_owned())
            .spawn(move || drain_stderr(stderr, stderr_target))
            .map_err(|error| format!("cannot start isolated worker stderr reader: {error}"))?;
        let mut process = Self {
            child,
            stdin,
            responses,
            stdout_thread: Some(stdout_thread),
            stderr_thread: Some(stderr_thread),
            stderr: captured_stderr,
            #[cfg(windows)]
            job,
        };
        let initialized = process.request(
            WorkerCommand::Initialize(Box::new(options.clone())),
            STARTUP_TIMEOUT,
            options.limits.max_resident_bytes,
        );
        match initialized {
            Ok(WorkerResponse::Initialized(Ok(()))) => Ok(process),
            Ok(WorkerResponse::Initialized(Err(message))) => {
                process.terminate();
                Err(format!("cannot initialize isolated Edge worker: {message}"))
            }
            Ok(_) => {
                process.terminate();
                Err("isolated worker returned an invalid initialization response".to_owned())
            }
            Err(message) => {
                process.terminate();
                Err(message)
            }
        }
    }

    fn request(
        &mut self,
        command: WorkerCommand,
        deadline: Duration,
        resident_limit: Option<usize>,
    ) -> Result<WorkerResponse, String> {
        write_frame(&mut self.stdin, &command, MAX_IPC_FRAME_BYTES).map_err(|error| {
            self.failure(format!("cannot send isolated worker request: {error}"))
        })?;
        let started = Instant::now();
        loop {
            if resident_limit.is_some_and(|limit| {
                process_resident_memory_bytes(&self.child)
                    .is_some_and(|resident| resident > limit as u64)
            }) {
                let limit = resident_limit.expect("resident limit was checked");
                return Err(self.failure(format!(
                    "isolated Edge worker exceeded max_resident_bytes ({limit} bytes)"
                )));
            }
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|error| self.failure(format!("cannot query isolated worker: {error}")))?
            {
                return Err(self.failure(format!(
                    "isolated Edge worker exited unexpectedly with {status}"
                )));
            }
            let elapsed = started.elapsed();
            if elapsed >= deadline {
                return Err(self.failure(format!(
                    "isolated Edge worker exceeded the wall-clock deadline of {} ms",
                    deadline.as_millis()
                )));
            }
            let remaining = deadline.saturating_sub(elapsed);
            match self.responses.recv_timeout(POLL_INTERVAL.min(remaining)) {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(message)) => {
                    return Err(
                        self.failure(format!("cannot read isolated worker response: {message}"))
                    );
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(
                        self.failure("isolated worker response channel disconnected".to_owned())
                    );
                }
            }
        }
    }

    fn failure(&mut self, message: String) -> String {
        self.terminate();
        let stderr = self.stderr_text();
        if stderr.is_empty() {
            message
        } else {
            format!("{message}: {stderr}")
        }
    }

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        self.stdin.flush().ok();
        if let Some(thread) = self.stdout_thread.take() {
            let _ = thread.join();
        }
        if let Some(thread) = self.stderr_thread.take() {
            let _ = thread.join();
        }
        #[cfg(windows)]
        {
            self.job.take();
        }
    }

    fn stderr_text(&self) -> String {
        let Ok(stderr) = self.stderr.lock() else {
            return String::new();
        };
        let mut text = String::from_utf8_lossy(&stderr.bytes).trim().to_owned();
        if stderr.truncated {
            text.push_str(" [stderr truncated]");
        }
        text
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

/// Runs the hidden worker protocol on stdin/stdout.
///
/// This is public only so applications embedding the library can dispatch the
/// hidden worker argument before starting their normal command-line parser.
#[doc(hidden)]
pub fn run_isolated_worker() -> Result<(), String> {
    if std::env::var_os(WORKER_ENVIRONMENT).as_deref() != Some(std::ffi::OsStr::new("1")) {
        return Err("isolated worker entry point requires its controller environment".to_owned());
    }
    apply_parent_death_policy();
    apply_network_policy();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    run_isolated_worker_protocol(stdin.lock(), stdout.lock())
}

fn run_isolated_worker_protocol(
    mut input: impl Read,
    mut output: impl Write,
) -> Result<(), String> {
    let command = read_frame::<_, WorkerCommand>(&mut input, MAX_IPC_FRAME_BYTES)
        .map_err(|error| format!("cannot read isolated worker initialization: {error}"))?;
    let WorkerCommand::Initialize(options) = command else {
        return Err("isolated worker expected an initialization command".to_owned());
    };
    let runtime = crate::EdgeRuntime::with_options(*options);
    match runtime {
        Ok(mut runtime) => {
            write_frame(
                &mut output,
                &WorkerResponse::Initialized(Ok(())),
                MAX_IPC_FRAME_BYTES,
            )
            .map_err(|error| format!("cannot acknowledge isolated worker startup: {error}"))?;
            loop {
                let command = match read_frame::<_, WorkerCommand>(&mut input, MAX_IPC_FRAME_BYTES)
                {
                    Ok(command) => command,
                    Err(message) if message.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(message) => {
                        return Err(format!("cannot read isolated worker command: {message}"));
                    }
                };
                let (response, shutdown) = execute_worker_command(&mut runtime, command);
                write_frame(&mut output, &response, MAX_IPC_FRAME_BYTES)
                    .map_err(|error| format!("cannot write isolated worker response: {error}"))?;
                if shutdown {
                    break;
                }
            }
            Ok(())
        }
        Err(message) => write_frame(
            &mut output,
            &WorkerResponse::Initialized(Err(message)),
            MAX_IPC_FRAME_BYTES,
        )
        .map_err(|error| format!("cannot report isolated worker startup failure: {error}")),
    }
}

/// Windows `rundll32.exe` entry used to start a worker from the DLL itself.
/// The controller sets a private environment marker and owns all stdio pipes.
#[cfg(windows)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn edge_sandbox_worker_entry(
    _window: *mut std::ffi::c_void,
    _instance: *mut std::ffi::c_void,
    _command_line: *const u8,
    _show: i32,
) {
    // rundll32 may invoke the export from a loader-managed callback stack.
    // V8 requires a conventional thread stack for its central-stack checks,
    // so the entire runtime lifetime is pinned to this dedicated thread.
    let status = match thread::Builder::new()
        .name("edge-sandbox-dll-worker".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(run_isolated_worker)
        .and_then(|worker| {
            worker
                .join()
                .map_err(|_| std::io::Error::other("isolated Edge worker thread panicked"))
        }) {
        Ok(Ok(())) => 0,
        Ok(Err(message)) => {
            eprintln!("{message}");
            70
        }
        Err(error) => {
            eprintln!("cannot run isolated Edge worker thread: {error}");
            71
        }
    };
    std::process::exit(status);
}

fn execute_worker_command(
    runtime: &mut crate::EdgeRuntime,
    command: WorkerCommand,
) -> (WorkerResponse, bool) {
    match command {
        WorkerCommand::Initialize(_) => (
            WorkerResponse::Unit(Err("isolated worker was already initialized".to_owned())),
            false,
        ),
        WorkerCommand::Evaluate { source, source_url } => (
            WorkerResponse::Evaluation(match source_url {
                Some(source_url) => runtime.evaluate_with_source_url(&source, &source_url),
                None => runtime.evaluate(&source),
            }),
            false,
        ),
        WorkerCommand::EnableProxyTrace => {
            (WorkerResponse::Unit(runtime.enable_proxy_trace()), false)
        }
        WorkerCommand::DisableProxyTrace => {
            runtime.disable_proxy_trace();
            (WorkerResponse::Unit(Ok(())), false)
        }
        WorkerCommand::ClearProxyTrace => {
            runtime.clear_proxy_trace();
            (WorkerResponse::Unit(Ok(())), false)
        }
        WorkerCommand::SetNativeTraceExclusions(exclusions) => (
            WorkerResponse::Unit(runtime.set_native_trace_exclusions(&exclusions)),
            false,
        ),
        WorkerCommand::ProxyTrace => (WorkerResponse::Trace(Ok(runtime.proxy_trace())), false),
        WorkerCommand::ProxyTraceMatching(needle) => (
            WorkerResponse::Trace(Ok(runtime.proxy_trace_matching(&needle))),
            false,
        ),
        WorkerCommand::NetworkRequests => (
            WorkerResponse::NetworkRequests(Ok(runtime.network_requests())),
            false,
        ),
        WorkerCommand::ClearNetworkRequests => {
            runtime.clear_network_requests();
            (WorkerResponse::Unit(Ok(())), false)
        }
        WorkerCommand::Stdout => (WorkerResponse::Stdout(Ok(runtime.stdout())), false),
        WorkerCommand::ClearStdout => {
            runtime.clear_stdout();
            (WorkerResponse::Unit(Ok(())), false)
        }
        WorkerCommand::Shutdown => (WorkerResponse::Unit(Ok(())), true),
    }
}

fn write_frame<W: Write, T: Serialize>(
    writer: &mut W,
    value: &T,
    maximum: usize,
) -> std::io::Result<()> {
    let bytes = bincode::serialize(value).map_err(std::io::Error::other)?;
    if bytes.len() > maximum {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("binary IPC frame exceeds {maximum} bytes"),
        ));
    }
    writer.write_all(&(bytes.len() as u64).to_le_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()
}

fn read_frame<R: Read, T: DeserializeOwned>(mut reader: R, maximum: usize) -> std::io::Result<T> {
    let mut length = [0_u8; 8];
    reader.read_exact(&mut length)?;
    let length = u64::from_le_bytes(length);
    if length > maximum as u64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("binary IPC frame exceeds {maximum} bytes"),
        ));
    }
    let mut bytes = vec![0_u8; length as usize];
    reader.read_exact(&mut bytes)?;
    bincode::deserialize(&bytes).map_err(std::io::Error::other)
}

#[derive(Default)]
struct LimitedBuffer {
    bytes: Vec<u8>,
    truncated: bool,
}

fn drain_stderr(mut stderr: impl Read, target: Arc<Mutex<LimitedBuffer>>) {
    let mut chunk = [0_u8; 4096];
    loop {
        let Ok(read) = stderr.read(&mut chunk) else {
            break;
        };
        if read == 0 {
            break;
        }
        let Ok(mut target) = target.lock() else {
            break;
        };
        let remaining = STDERR_CAPTURE_BYTES.saturating_sub(target.bytes.len());
        target
            .bytes
            .extend_from_slice(&chunk[..read.min(remaining)]);
        target.truncated |= read > remaining;
    }
}

fn preserve_runtime_environment(command: &mut Command) {
    for name in [
        "SystemRoot",
        "WINDIR",
        "TEMP",
        "TMP",
        "LD_LIBRARY_PATH",
        "DYLD_LIBRARY_PATH",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
}

#[cfg(target_os = "linux")]
fn apply_parent_death_policy() {
    // SAFETY: PR_SET_PDEATHSIG only asks the kernel to terminate this worker
    // if its controller exits; it does not grant capabilities.
    unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
    }
}

#[cfg(not(target_os = "linux"))]
fn apply_parent_death_policy() {}

#[cfg(target_os = "linux")]
fn apply_network_policy() {
    const DENY: u32 = libc::SECCOMP_RET_ERRNO | libc::EPERM as u32;
    let mut filter = vec![libc::sock_filter {
        code: (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16,
        jt: 0,
        jf: 0,
        k: 0,
    }];
    for syscall in [
        libc::SYS_socket,
        libc::SYS_connect,
        libc::SYS_bind,
        libc::SYS_listen,
        libc::SYS_accept,
        libc::SYS_accept4,
        libc::SYS_sendto,
        libc::SYS_recvfrom,
    ] {
        filter.push(libc::sock_filter {
            code: (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
            jt: 0,
            jf: 1,
            k: syscall as u32,
        });
        filter.push(libc::sock_filter {
            code: (libc::BPF_RET | libc::BPF_K) as u16,
            jt: 0,
            jf: 0,
            k: DENY,
        });
    }
    filter.push(libc::sock_filter {
        code: (libc::BPF_RET | libc::BPF_K) as u16,
        jt: 0,
        jf: 0,
        k: libc::SECCOMP_RET_ALLOW,
    });
    let program = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_mut_ptr(),
    };
    // SAFETY: no_new_privs plus this BPF program can only remove network
    // syscalls from the already-separated worker process.
    unsafe {
        if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) == 0 {
            libc::prctl(
                libc::PR_SET_SECCOMP,
                libc::SECCOMP_MODE_FILTER,
                &program as *const libc::sock_fprog,
            );
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn apply_network_policy() {}

#[cfg(target_os = "linux")]
fn process_resident_memory_bytes(child: &WorkerChild) -> Option<u64> {
    let status = std::fs::read_to_string(format!("/proc/{}/status", child.id())).ok()?;
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    line.split_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()
        .map(|kib| kib.saturating_mul(1024))
}

#[cfg(target_os = "macos")]
fn process_resident_memory_bytes(child: &WorkerChild) -> Option<u64> {
    let mut info = std::mem::MaybeUninit::<libc::proc_taskinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_taskinfo>();
    // SAFETY: proc_pidinfo writes at most `size` bytes to this valid buffer.
    let read = unsafe {
        libc::proc_pidinfo(
            child.id() as libc::c_int,
            libc::PROC_PIDTASKINFO,
            0,
            info.as_mut_ptr().cast(),
            size as libc::c_int,
        )
    };
    if read != size as libc::c_int {
        return None;
    }
    // SAFETY: a full-size successful read initialized the structure.
    Some(unsafe { info.assume_init() }.pti_resident_size)
}

#[cfg(windows)]
fn process_resident_memory_bytes(child: &WorkerChild) -> Option<u64> {
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct ProcessMemoryCounters {
        size: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[link(name = "psapi")]
    unsafe extern "system" {
        fn GetProcessMemoryInfo(
            process: *mut std::ffi::c_void,
            counters: *mut ProcessMemoryCounters,
            size: u32,
        ) -> i32;
    }

    let mut counters = ProcessMemoryCounters {
        size: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        page_fault_count: 0,
        peak_working_set_size: 0,
        working_set_size: 0,
        quota_peak_paged_pool_usage: 0,
        quota_paged_pool_usage: 0,
        quota_peak_non_paged_pool_usage: 0,
        quota_non_paged_pool_usage: 0,
        pagefile_usage: 0,
        peak_pagefile_usage: 0,
    };
    // SAFETY: the child process handle is valid while `child` is alive and the
    // counter buffer matches the size passed to GetProcessMemoryInfo.
    let succeeded = unsafe {
        GetProcessMemoryInfo(
            child.spawned().as_raw_handle(),
            &mut counters,
            counters.size,
        )
    };
    (succeeded != 0).then_some(counters.working_set_size as u64)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn process_resident_memory_bytes(_child: &WorkerChild) -> Option<u64> {
    None
}

#[cfg(windows)]
struct WindowsJob(isize);

#[cfg(windows)]
impl WindowsJob {
    fn attach(child: &WorkerChild, resident_limit: Option<usize>) -> Result<Self, String> {
        use std::os::windows::io::AsRawHandle;

        const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: i32 = 9;
        const JOB_OBJECT_LIMIT_ACTIVE_PROCESS: u32 = 0x0000_0008;
        const JOB_OBJECT_LIMIT_PROCESS_MEMORY: u32 = 0x0000_0100;
        const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;

        #[repr(C)]
        #[derive(Default)]
        struct BasicLimitInformation {
            per_process_user_time_limit: i64,
            per_job_user_time_limit: i64,
            limit_flags: u32,
            minimum_working_set_size: usize,
            maximum_working_set_size: usize,
            active_process_limit: u32,
            affinity: usize,
            priority_class: u32,
            scheduling_class: u32,
        }

        #[repr(C)]
        #[derive(Default)]
        struct IoCounters {
            read_operation_count: u64,
            write_operation_count: u64,
            other_operation_count: u64,
            read_transfer_count: u64,
            write_transfer_count: u64,
            other_transfer_count: u64,
        }

        #[repr(C)]
        #[derive(Default)]
        struct ExtendedLimitInformation {
            basic_limit_information: BasicLimitInformation,
            io_info: IoCounters,
            process_memory_limit: usize,
            job_memory_limit: usize,
            peak_process_memory_used: usize,
            peak_job_memory_used: usize,
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn CreateJobObjectW(
                attributes: *const std::ffi::c_void,
                name: *const u16,
            ) -> *mut std::ffi::c_void;
            fn SetInformationJobObject(
                job: *mut std::ffi::c_void,
                information_class: i32,
                information: *const std::ffi::c_void,
                information_length: u32,
            ) -> i32;
            fn AssignProcessToJobObject(
                job: *mut std::ffi::c_void,
                process: *mut std::ffi::c_void,
            ) -> i32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }

        // SAFETY: null attributes and name create an unnamed job owned solely
        // by this controller.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(format!(
                "cannot create isolated worker job object: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mut limits = ExtendedLimitInformation::default();
        limits.basic_limit_information.limit_flags =
            JOB_OBJECT_LIMIT_ACTIVE_PROCESS | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        limits.basic_limit_information.active_process_limit = 1;
        if let Some(maximum) = resident_limit {
            limits.basic_limit_information.limit_flags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
            limits.process_memory_limit = maximum;
        }
        // SAFETY: `limits` has the documented extended-limit layout and the
        // child handle remains valid for the assignment call.
        let configured = unsafe {
            SetInformationJobObject(
                job,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
                (&limits as *const ExtendedLimitInformation).cast(),
                std::mem::size_of::<ExtendedLimitInformation>() as u32,
            )
        };
        let assigned = if configured != 0 {
            // SAFETY: both handles are live and owned by this controller.
            unsafe { AssignProcessToJobObject(job, child.spawned().as_raw_handle()) }
        } else {
            0
        };
        if configured == 0 || assigned == 0 {
            let error = std::io::Error::last_os_error();
            // SAFETY: `job` is a valid handle created above.
            unsafe {
                CloseHandle(job);
            }
            return Err(format!(
                "cannot constrain isolated worker with a job object: {error}"
            ));
        }
        Ok(Self(job as isize))
    }
}

#[cfg(windows)]
impl Drop for WindowsJob {
    fn drop(&mut self) {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }
        // SAFETY: this handle is uniquely owned and closed exactly once.
        unsafe {
            CloseHandle(self.0 as *mut std::ffi::c_void);
        }
    }
}
