use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

/// An opencode instance discovered on the system.
#[derive(Debug, Clone, Serialize)]
pub struct Instance {
    pub pid: u32,
    pub name: String,
    pub project: Option<String>,
    pub state: ProcessState,
    pub memory_kb: u64,
    pub uptime_secs: u64,
    pub serve_port: Option<u16>,
}

impl Instance {
    pub fn is_serve(&self) -> bool {
        self.serve_port.is_some()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Stopped,
    TracingStop,
    Zombie,
    Dead,
    Wakekill,
    Waking,
    Parked,
    Idle,
    Lock,
    Unknown(char),
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Sleeping => write!(f, "sleeping"),
            Self::DiskSleep => write!(f, "disk-sleep"),
            Self::Stopped => write!(f, "stopped"),
            Self::TracingStop => write!(f, "trace-stop"),
            Self::Zombie => write!(f, "zombie"),
            Self::Dead => write!(f, "dead"),
            Self::Wakekill => write!(f, "wakekill"),
            Self::Waking => write!(f, "waking"),
            Self::Parked => write!(f, "parked"),
            Self::Idle => write!(f, "idle"),
            Self::Lock => write!(f, "lock"),
            Self::Unknown(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceStats {
    pub sessions: u64,
    pub messages: u64,
    pub total_cost: f64,
    pub avg_cost_per_day: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
}

/// Discover all opencode instances by scanning /proc.
pub fn discover_all() -> io::Result<Vec<Instance>> {
    let mut instances = Vec::new();

    let proc = fs::read_dir("/proc")?;
    for entry in proc {
        let entry = entry?;
        let pid_str = entry.file_name();
        let pid: u32 = match pid_str.to_str().and_then(|s| s.parse().ok()) {
            Some(p) => p,
            None => continue,
        };

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let comm_path = entry.path().join("comm");
        let comm = match fs::read_to_string(&comm_path) {
            Ok(c) => c.trim().to_string(),
            Err(_) => continue,
        };

        // Only match opencode processes
        if comm != "opencode" {
            continue;
        }

        match read_instance(pid, &entry.path()) {
            Ok(inst) => instances.push(inst),
            Err(e) => {
                eprintln!("  warn: pid {pid}: {e}");
            }
        }
    }

    Ok(instances)
}

fn read_instance(pid: u32, proc_path: &Path) -> io::Result<Instance> {
    let cmdline_raw = fs::read(proc_path.join("cmdline"))?;
    let cmdline = parse_cmdline(&cmdline_raw);

    let status_text = fs::read_to_string(proc_path.join("status"))?;
    let status = parse_status(&status_text);

    let state_code = status.state;
    let state = parse_state(state_code);

    let name = cmdline
        .first()
        .map(|s| {
            Path::new(s)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("opencode")
                .to_string()
        })
        .unwrap_or_else(|| "opencode".to_string());

    let project = cmdline
        .iter()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .cloned()
        .or_else(|| resolve_cwd(proc_path).ok());

    let serve_port = cmdline
        .windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse::<u16>().ok())
        .or_else(|| {
            cmdline
                .iter()
                .find(|a| a.starts_with("--port="))
                .and_then(|a| a.split('=').nth(1))
                .and_then(|p| p.parse().ok())
        });

    let boot_time = read_boot_time()?;
    let clock_ticks = 100; // standard USER_HZ on Linux
    let stat_text = fs::read_to_string(proc_path.join("stat"))?;
    let fields = parse_stat_fields(&stat_text);

    let uptime_secs = {
        let starttime = fields
            .flat_get(21)
            .and_then(|s: &str| s.parse::<u64>().ok())
            .unwrap_or(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let start_secs = boot_time + starttime / clock_ticks;
        now.saturating_sub(start_secs)
    };

    Ok(Instance {
        pid,
        name,
        project,
        state,
        memory_kb: status.memory_kb,
        uptime_secs,
        serve_port,
    })
}

fn parse_cmdline(raw: &[u8]) -> Vec<String> {
    raw.split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .filter_map(|s| String::from_utf8(s.to_vec()).ok())
        .collect()
}

struct ParsedStatus {
    state: char,
    memory_kb: u64,
}

fn parse_status(text: &str) -> ParsedStatus {
    let mut state = '?';
    let mut memory_kb = 0u64;
    for line in text.lines() {
        if let Some(val) = line.strip_prefix("State:\t") {
            state = val.chars().next().unwrap_or('?');
        } else if let Some(val) = line.strip_prefix("VmRSS:\t") {
            // strip " kB" suffix, then trim whitespace for parse
            let trimmed = val.trim_end_matches(" kB").trim();
            memory_kb = trimmed.parse().unwrap_or(0);
        }
    }
    ParsedStatus { state, memory_kb }
}

fn parse_state(c: char) -> ProcessState {
    match c {
        'R' => ProcessState::Running,
        'S' => ProcessState::Sleeping,
        'D' => ProcessState::DiskSleep,
        'T' => ProcessState::Stopped,
        't' => ProcessState::TracingStop,
        'Z' => ProcessState::Zombie,
        'X' => ProcessState::Dead,
        'K' => ProcessState::Wakekill,
        'W' => ProcessState::Waking,
        'P' => ProcessState::Parked,
        'I' => ProcessState::Idle,
        'L' => ProcessState::Lock,
        c => ProcessState::Unknown(c),
    }
}

fn resolve_cwd(proc_path: &Path) -> io::Result<String> {
    let cwd = fs::read_link(proc_path.join("cwd"))?;
    Ok(cwd.to_string_lossy().into_owned())
}

fn read_boot_time() -> io::Result<u64> {
    let bt = fs::read_to_string("/proc/stat")?;
    for line in bt.lines() {
        if let Some(val) = line.strip_prefix("btime ") {
            return val
                .trim()
                .parse::<u64>()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "btime not found"))
}

/// Minimal parser for /proc/[pid]/stat fields (space-delimited, parens-surround comm).
fn parse_stat_fields(text: &str) -> StatFields<'_> {
    // comm is in parens and may contain spaces/parens — find the last ')'
    let close_paren = match text.rfind(") ") {
        Some(i) => i,
        None => {
            return StatFields {
                text,
                after_paren: 0,
            };
        }
    };
    StatFields {
        text,
        after_paren: close_paren + 2,
    }
}

struct StatFields<'a> {
    text: &'a str,
    after_paren: usize,
}

impl<'a> StatFields<'a> {
    fn flat_get(&self, index: usize) -> Option<&'a str> {
        // fields 1 and 2 are pid and comm — after_paren starts at field 3
        self.text[self.after_paren..]
            .split_whitespace()
            .nth(index.saturating_sub(3))
    }
}
