use nub_sandbox::Prepared;
use std::io::Read;
use std::process::Output;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

fn read(mut pipe: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes).expect("tool output drains");
    bytes
}

// A stuck native tool must fail its own case, not consume the entire OS matrix.
// The caller requests piped streams through CommandSpec's redaction flags.
pub fn output(prepared: Prepared) -> Output {
    let mut child = prepared.spawn().expect("tool command launches");
    let stdout = child.take_stdout().expect("tool stdout is piped");
    let stderr = child.take_stderr().expect("tool stderr is piped");
    let cancelled = AtomicBool::new(false);
    let (done, deadline) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let cancelled = &cancelled;
        let timer = scope.spawn(move || {
            if deadline.recv_timeout(Duration::from_secs(30)).is_err() {
                cancelled.store(true, Ordering::Release);
            }
        });
        let stdout = scope.spawn(move || read(stdout));
        let stderr = scope.spawn(move || read(stderr));
        let status = child.wait_cancellable(cancelled);
        drop(child);
        let _ = done.send(());
        timer.join().expect("tool deadline thread joins");
        let stdout = stdout.join().expect("tool stdout thread joins");
        let stderr = stderr.join().expect("tool stderr thread joins");
        let status = status.unwrap_or_else(|error| {
            panic!(
                "tool failed or exceeded its 30-second deadline: {error}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            )
        });
        Output {
            status,
            stdout,
            stderr,
        }
    })
}
