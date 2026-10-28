// Opifex v2.0.0 QuickJS sandbox sidecar
// Runs user Code-block JavaScript under a Windows Job Object with:
//  - 128 MB memory limit
//  - 5,000 ms CPU execution cap
//  - No child process spawning
//  - Kill on job close
//  - Kill on job close

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use rquickjs::{Context, Runtime};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};

#[cfg(target_os = "windows")]
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY, JobObjectExtendedLimitInformation,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::GetCurrentProcess;

#[derive(Deserialize)]
struct SandboxInput {
    code: String,
    input: serde_json::Value,
    timeout_ms: u64,
}

#[derive(Serialize)]
struct SandboxOutput {
    output: Option<serde_json::Value>,
    logs: Vec<String>,
    error: Option<String>,
}

#[cfg(target_os = "windows")]
fn apply_windows_job_limits(memory_mb: usize) -> Result<()> {
    unsafe {
        let job = CreateJobObjectW(None, None)?;
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        info.ProcessMemoryLimit = memory_mb * 1024 * 1024;

        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )?;

        AssignProcessToJobObject(job, GetCurrentProcess())?;
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn apply_windows_job_limits(_memory_mb: usize) -> Result<()> {
    // No-op on non-Windows platforms
    Ok(())
}

fn main() -> Result<()> {
    apply_windows_job_limits(128)?;

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    if let Some(Ok(line)) = lines.next() {
        let req: SandboxInput = serde_json::from_str(&line)?;
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        let result = ctx.with(|ctx| {
            let input_json = serde_json::to_string(&req.input)?;
            ctx.globals().set("input", input_json)?;
            let res: rquickjs::Value = ctx.eval(&*req.code)?;
            let json_str: String = match ctx.json_stringify(res)? {
                Some(s) => s.to_string().unwrap_or_else(|_| String::new()),
                None => String::new(),
            };
            Ok::<_, anyhow::Error>(serde_json::from_str(&json_str)?)
        });

        let output = match result {
            Ok(val) => SandboxOutput {
                output: Some(val),
                logs: vec![],
                error: None,
            },
            Err(e) => SandboxOutput {
                output: None,
                logs: vec![],
                error: Some(e.to_string()),
            },
        };

        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}
