use std::io::Read;
use std::process::ExitCode;

use edge_sandbox::{EdgeRuntimeOptions, IsolatedEdgeRuntime};

fn main() -> ExitCode {
    let mut arguments = std::env::args();
    let _program = arguments.next();
    let command = arguments.next().unwrap_or_else(|| "run".to_owned());

    match command.as_str() {
        "__edge_sandbox_worker" => isolated_worker(),
        "run" => run(arguments.collect()),
        "window-names" => window_names(),
        "--help" | "-h" | "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("unknown command: {command}");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn run(arguments: Vec<String>) -> ExitCode {
    let trace = arguments.iter().any(|argument| argument == "--trace");
    let export_requests = arguments.iter().any(|argument| argument == "--requests");
    let trace_filter = arguments
        .iter()
        .find_map(|argument| argument.strip_prefix("--trace-filter="));
    let paths = arguments
        .iter()
        .filter(|argument| {
            argument.as_str() != "--trace"
                && argument.as_str() != "--requests"
                && !argument.starts_with("--trace-filter=")
        })
        .collect::<Vec<_>>();
    let source = if paths.is_empty() || (paths.len() == 1 && paths[0].as_str() == "-") {
        let mut source = String::new();
        if let Err(error) = std::io::stdin().read_to_string(&mut source) {
            eprintln!("cannot read stdin: {error}");
            return ExitCode::from(2);
        }
        source
    } else {
        match std::fs::read_to_string(paths[0]) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("cannot read {}: {error}", paths[0]);
                return ExitCode::from(2);
            }
        }
    };

    let runtime = match IsolatedEdgeRuntime::new(EdgeRuntimeOptions::default()) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("cannot initialize Edge runtime: {error}");
            return ExitCode::from(1);
        }
    };
    if trace && let Err(error) = runtime.enable_native_trace() {
        eprintln!("cannot enable native trace: {error}");
        return ExitCode::from(1);
    }
    match runtime.evaluate(&source) {
        Ok(value) => {
            println!("{value}");
            if trace {
                let entries = match trace_filter {
                    Some(filter) => runtime.native_trace_matching(filter),
                    None => runtime.native_trace(),
                };
                match entries {
                    Ok(entries) => {
                        for entry in entries {
                            println!("{entry}");
                        }
                    }
                    Err(error) => {
                        eprintln!("cannot read native trace: {error}");
                        return ExitCode::from(1);
                    }
                }
            }
            if export_requests {
                match runtime.network_requests() {
                    Ok(entries) => print_network_requests(&entries),
                    Err(error) => {
                        eprintln!("cannot read captured network requests: {error}");
                        return ExitCode::from(1);
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn window_names() -> ExitCode {
    let runtime = match IsolatedEdgeRuntime::new(EdgeRuntimeOptions::default()) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("cannot initialize Edge runtime: {error}");
            return ExitCode::from(1);
        }
    };
    let source = "Object.getOwnPropertyNames(globalThis).map((name, index) => `${index}\\t${name}`).join('\\n')";
    match runtime.evaluate(source) {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn isolated_worker() -> ExitCode {
    match edge_sandbox::run_isolated_worker() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(70)
        }
    }
}

fn print_help() {
    println!("edge-sandbox run [FILE|-] [--requests] [--trace] [--trace-filter=TEXT]");
    println!("edge-sandbox window-names");
}

fn print_network_requests(entries: &[edge_sandbox::CapturedNetworkRequest]) {
    for entry in entries {
        let source = match entry.source {
            edge_sandbox::NetworkRequestSource::XmlHttpRequest => "XMLHttpRequest",
            edge_sandbox::NetworkRequestSource::Fetch => "fetch",
        };
        println!(
            "REQUEST\t{}\t{}\t{}\t{}",
            entry.sequence,
            source,
            escape_field(&entry.method),
            escape_field(&entry.url)
        );
        for (name, value) in &entry.headers {
            println!("HEADER\t{}\t{}", escape_field(name), escape_field(value));
        }
        println!("BODY\t{}\t{}", entry.body.len(), encode_hex(&entry.body));
        println!("END_REQUEST");
    }
}

fn escape_field(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect(),
            '\r' => "\\r".chars().collect(),
            '\n' => "\\n".chars().collect(),
            character => vec![character],
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}
