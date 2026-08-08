use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

#[test]
fn worker_timers_use_delay_order_microtasks_intervals_and_string_handlers() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            const order = [];
            queueMicrotask(() => order.push("microtask"));
            setTimeout(() => order.push("ten-a"), 10);
            setTimeout(() => order.push("five"), 5);
            setTimeout(() => {
              order.push("ten-b");
              postMessage("ORDER:" + order.join(","));
            }, 10);
            let count = 0;
            const interval = setInterval(() => {
              count++;
              if (count === 3) {
                clearInterval(interval);
                postMessage("INTERVAL:" + count);
              }
            }, 7);
            setTimeout("postMessage('STRING:true')", 12);
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerTimerAnswers = [];
          worker.onmessage = event => workerTimerAnswers.push(event.data);
        })()
        "#,
    );
    let answers = text(&mut runtime, "workerTimerAnswers.slice().sort().join('|')");
    assert_eq!(
        answers,
        "INTERVAL:3|ORDER:microtask,five,ten-a,ten-b|STRING:true"
    );
}

#[test]
fn worker_clock_uses_its_performance_time_origin_and_real_timer_deadlines() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            const start = performance.now();
            const wallStart = Date.now();
            const eventStamp = new Event("worker-clock").timeStamp;
            setTimeout(() => {
              const end = performance.now();
              postMessage([
                end - start,
                Date.now() - wallStart,
                Math.abs(performance.timeOrigin + end - Date.now()),
                Math.abs(eventStamp - start),
                Function.prototype.toString.call(performance.now)
              ].join("|"));
            }, 15);
          `;
          globalThis.workerClockAnswer = "pending";
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          worker.onmessage = event => workerClockAnswer = event.data;
        })()
        "#,
    );
    let values = text(&mut runtime, "workerClockAnswer")
        .split('|')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 5);
    let timer_delta = values[0].parse::<f64>().expect("worker timer delta");
    let date_delta = values[1].parse::<f64>().expect("worker Date delta");
    let epoch_skew = values[2].parse::<f64>().expect("worker epoch skew");
    let event_skew = values[3].parse::<f64>().expect("worker Event skew");
    assert!(timer_delta >= 14.0, "worker timer delta was {timer_delta}");
    assert!(date_delta >= 10.0, "worker Date delta was {date_delta}");
    assert!(epoch_skew <= 5.0, "worker epoch skew was {epoch_skew}");
    assert!(event_skew <= 2.0, "worker Event skew was {event_skew}");
    assert_eq!(values[4], "function now() { [native code] }");
}

#[test]
fn worker_close_keeps_sent_message_and_discards_later_tasks() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            onmessage = () => {
              postMessage("before-close");
              setTimeout(() => postMessage("timer-after-close"), 0);
              close();
              postMessage("after-close");
            };
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerCloseAnswers = [];
          worker.onmessage = event => workerCloseAnswers.push(event.data);
          worker.postMessage("run");
        })()
        "#,
    );
    assert_eq!(
        text(&mut runtime, "workerCloseAnswers.join('|')"),
        "before-close"
    );
}

#[test]
fn worker_message_port_transfer_is_synchronous_and_cross_realm() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            onmessage = event => {
              const port = event.data.port;
              port.onmessage = nested =>
                port.postMessage("pong:" + nested.data);
              postMessage(
                "status:" + [
                  event.ports[0] === port,
                  port instanceof MessagePort
                ].join(",")
              );
            };
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          const channel = new MessageChannel();
          globalThis.workerPortAnswers = [];
          worker.onmessage = event => workerPortAnswers.push(event.data);
          channel.port1.onmessage = event =>
            workerPortAnswers.push(event.data);
          worker.postMessage(
            { port: channel.port2 },
            { transfer: [channel.port2] }
          );
          let detached = "none";
          try {
            channel.port2.postMessage("invalid");
          } catch (error) {
            detached = error.name;
          }
          workerPortAnswers.push("detached:" + detached);
          channel.port1.postMessage("hi");
        })()
        "#,
    );
    assert_eq!(
        text(&mut runtime, "workerPortAnswers.slice().sort().join('|')"),
        "detached:DataCloneError|pong:hi|status:true,true"
    );
}

#[test]
fn nested_worker_messages_return_to_the_creating_worker_realm() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            const nestedSource = "postMessage(42)";
            const nested = new Worker(
              "data:text/javascript," + encodeURIComponent(nestedSource)
            );
            nested.onmessage = event => postMessage([
              event.data,
              nested instanceof Worker,
              Object.getPrototypeOf(nested) === Worker.prototype
            ]);
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.nestedWorkerAnswer = "pending";
          worker.onmessage = event =>
            nestedWorkerAnswer = event.data.join("|");
        })()
        "#,
    );
    assert_eq!(text(&mut runtime, "nestedWorkerAnswer"), "42|true|true");
}
