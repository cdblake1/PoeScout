use std::path::PathBuf;
use std::process::Command;

/// Common PoB install locations on Windows.
const POB_SEARCH_PATHS: &[&str] = &[
    r"C:\ProgramData\Path of Building\Path of Building.exe",
    r"C:\Program Files (x86)\Path of Building\Path of Building.exe",
    r"C:\Program Files\Path of Building\Path of Building.exe",
];

/// Detect PoB install path by checking common locations.
pub fn detect_pob_path() -> Option<PathBuf> {
    // Check common install paths
    for path in POB_SEARCH_PATHS {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Check PATH
    if let Ok(output) = Command::new("where").arg("Path of Building.exe").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let first_line = path_str.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() {
                let p = PathBuf::from(first_line);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    None
}

/// Launch PoB with an optional build code argument.
/// Returns Ok(()) if the process was spawned, Err with reason otherwise.
pub fn launch_pob(pob_path: &PathBuf, build_code: Option<&str>) -> Result<(), String> {
    if !pob_path.exists() {
        return Err(format!("PoB not found at: {}", pob_path.display()));
    }

    let mut cmd = Command::new(pob_path);

    if let Some(code) = build_code {
        // PoB Community Fork accepts build codes via command line
        cmd.arg(code);
    }

    cmd.spawn()
        .map(|_| ())
        .map_err(|e| format!("Failed to launch PoB: {}", e))
}
