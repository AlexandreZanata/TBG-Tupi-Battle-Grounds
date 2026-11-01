//! The launcher helper as a child process (P66 §3.1): one command at a
//! time, JSON lines read on a thread and handed to Bevy through a channel,
//! `cancel` written to its stdin. No network or file work runs in the
//! render loop.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;

/// Where the launcher put this managed install.
#[derive(Clone, Debug)]
pub struct Managed {
    pub helper: PathBuf,
    pub root: PathBuf,
    pub build: String,
    /// Health file and nonce for this launch (a candidate must report).
    pub health: Option<(PathBuf, String)>,
}

impl Managed {
    /// Launched by `tbg-launcher`? A direct or legacy run is unmanaged.
    pub fn from_env() -> Option<Managed> {
        let helper = PathBuf::from(std::env::var_os("TBG_UPDATE_HELPER")?);
        let root = PathBuf::from(std::env::var_os("TBG_UPDATE_ROOT")?);
        let build = std::env::var("TBG_INSTALLED_BUILD").ok()?;
        let health = std::env::var_os("TBG_HEALTH_FILE")
            .zip(std::env::var("TBG_HEALTH_NONCE").ok())
            .map(|(f, n)| (PathBuf::from(f), n));
        Some(Managed {
            helper,
            root,
            build,
            health,
        })
    }
}

/// One running helper command.
pub struct Running {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    /// Behind a mutex so the owning resource is `Sync` (Bevy resources).
    events: Mutex<Receiver<serde_json::Value>>,
}

impl Running {
    pub fn start(m: &Managed, args: &[&str]) -> std::io::Result<Running> {
        let mut child = Command::new(&m.helper)
            .arg("helper")
            .args(args)
            .env("TBG_UPDATE_ROOT", &m.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("no helper stdout"))?;
        let (tx, events) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout)
                .lines()
                .map_while(Result::ok)
                .take(10_000)
            {
                if line.len() > 1 << 20 {
                    break;
                }
                if let Ok(v) = serde_json::from_str(&line) {
                    if tx.send(v).is_err() {
                        break;
                    }
                }
            }
        });
        let stdin = child.stdin.take();
        Ok(Running {
            child: Some(child),
            stdin,
            events: Mutex::new(events),
        })
    }

    /// Events delivered so far.
    pub fn drain(&self) -> Vec<serde_json::Value> {
        let Ok(rx) = self.events.lock() else {
            return Vec::new();
        };
        std::iter::from_fn(|| rx.try_recv().ok()).collect()
    }

    /// Ask a download to stop at its next chunk.
    pub fn cancel(&mut self) {
        if let Some(stdin) = self.stdin.as_mut() {
            let _ = writeln!(stdin, "cancel");
        }
        self.stdin = None;
    }

    /// The helper exited (its last events may still be queued).
    pub fn finished(&mut self) -> bool {
        self.child
            .as_mut()
            .is_none_or(|c| matches!(c.try_wait(), Ok(Some(_)) | Err(_)))
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        // Closing stdin cancels a download; reap the helper off-thread.
        self.stdin = None;
        if let Some(mut child) = self.child.take() {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
    }
}
