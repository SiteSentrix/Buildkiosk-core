//! Starts the program under test (a `buildkiosk` binary, or a build that wraps the core): the command line (`demo`, `token`, `check`) and `serve` on a free local
//! port with a throwaway data folder, its output kept in a log file.

use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Runs a command-line subcommand; its standard output, or everything it printed when it fails.
pub fn cli(bin: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new(bin).args(args).env("RUST_LOG", "warn").stdin(Stdio::null()).output().map_err(|e| format!("{}: {e}", bin.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if out.status.success() {
        Ok(stdout)
    } else {
        Err(format!("`buildkiosk {}` failed ({}): {stdout}{}", args.join(" "), out.status, String::from_utf8_lossy(&out.stderr)))
    }
}

pub struct Server {
    child: Child,
    pub port: u16,
    pub log: PathBuf,
}

impl Server {
    pub fn start(bin: &Path, data: &Path, extra: &[&str], log: &Path) -> Result<Server, String> {
        let port = TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).map_err(|e| format!("no free port: {e}"))?.port();
        let out = File::create(log).map_err(|e| format!("{}: {e}", log.display()))?;
        let err = out.try_clone().map_err(|e| e.to_string())?;
        let data = data.to_str().ok_or("the data folder has an odd name")?;
        let bind = format!("127.0.0.1:{port}");
        let child = Command::new(bin)
            .args(["serve", "--data", data, "--bind", bind.as_str()])
            .args(extra)
            .env("RUST_LOG", "info")
            .env("RUST_BACKTRACE", "1")
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err)
            .spawn()
            .map_err(|e| format!("{}: {e}", bin.display()))?;
        let mut server = Server { child, port, log: log.to_path_buf() };
        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return Ok(server);
            }
            if let Ok(Some(status)) = server.child.try_wait() {
                return Err(format!("the server stopped ({status}): {}", server.log_text()));
            }
            if Instant::now() > deadline {
                return Err(format!("the server did not answer within 90 s: {}", server.log_text()));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn log_text(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }

    /// Stops the server; whether it was still running (a crash would have ended it already).
    pub fn stop(&mut self) -> bool {
        let running = matches!(self.child.try_wait(), Ok(None));
        let _ = self.child.kill();
        let _ = self.child.wait();
        running
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}
