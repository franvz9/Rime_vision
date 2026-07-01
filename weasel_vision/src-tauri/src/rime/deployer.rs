use anyhow::Result;

#[cfg(target_os = "windows")]
fn find_weasel_deployer() -> Option<std::path::PathBuf> {
    let mut deployer_paths: Vec<std::path::PathBuf> = Vec::new();

    // Method 1: Use environment variables - check Rime\* subdirectories
    if let Ok(pf) = std::env::var("ProgramFiles") {
        let rime_base = std::path::PathBuf::from(&pf).join("Rime");
        if rime_base.exists() {
            deployer_paths.push(rime_base.join("WeaselDeployer.exe"));
            if let Ok(entries) = std::fs::read_dir(&rime_base) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let subdir_deployer = entry.path().join("WeaselDeployer.exe");
                        if subdir_deployer.exists() {
                            deployer_paths.push(subdir_deployer);
                        }
                    }
                }
            }
        }
    }
    if let Ok(pfx86) = std::env::var("ProgramFiles(x86)") {
        let rime_base = std::path::PathBuf::from(&pfx86).join("Rime");
        if rime_base.exists() {
            deployer_paths.push(rime_base.join("WeaselDeployer.exe"));
            if let Ok(entries) = std::fs::read_dir(&rime_base) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let subdir_deployer = entry.path().join("WeaselDeployer.exe");
                        if subdir_deployer.exists() {
                            deployer_paths.push(subdir_deployer);
                        }
                    }
                }
            }
        }
    }

    // Method 2: Fallback to common default paths
    deployer_paths.push(std::path::PathBuf::from("C:\\Program Files\\Rime\\WeaselDeployer.exe"));
    deployer_paths.push(std::path::PathBuf::from("C:\\Program Files (x86)\\Rime\\WeaselDeployer.exe"));

    // Method 3: Check user's AppData\Local\Programs
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        deployer_paths.push(std::path::PathBuf::from(&local_appdata).join("Programs\\Rime\\WeaselDeployer.exe"));
    }

    for path in &deployer_paths {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

pub fn deploy() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Use pkill for atomic process matching and signaling (avoids TOCTOU race condition)
        // -x matches exact process name, -HUP sends reload signal
        // Exit code 0 = signal sent, 1 = no matching process (both are acceptable)
        match std::process::Command::new("pkill")
            .args(["-HUP", "-x", "Squirrel"])
            .output()
        {
            Ok(output) => {
                if !output.status.success() && output.status.code() != Some(1) {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("WeaselVision: pkill returned unexpected error: {}", stderr);
                }
            }
            Err(e) => {
                eprintln!("WeaselVision: failed to execute pkill: {}", e);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let deployer_path = find_weasel_deployer()
            .ok_or_else(|| anyhow::anyhow!("WeaselDeployer.exe not found. Is Weasel installed?"))?;

        eprintln!("WeaselVision: Found WeaselDeployer at: {:?}", deployer_path);
        let output = std::process::Command::new(&deployer_path)
            .arg("/deploy")
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("WeaselDeployer /deploy failed: {}", stderr);
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("Deploy is only supported on macOS and Windows");
    }

    Ok(())
}

pub fn sync() -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        let deployer_path = find_weasel_deployer()
            .ok_or_else(|| anyhow::anyhow!("WeaselDeployer.exe not found"))?;

        eprintln!("WeaselVision: Found WeaselDeployer at: {:?}", deployer_path);
        let output = std::process::Command::new(&deployer_path)
            .arg("/sync")
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("WeaselDeployer /sync failed: {}", stderr);
        }
        return Ok(true);
    }

    #[cfg(target_os = "macos")]
    {
        // Check if Squirrel is running before sending signal
        let is_running = std::process::Command::new("pkill")
            .args(["-0", "-x", "Squirrel"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if is_running {
            deploy()?;
            Ok(true)
        } else {
            eprintln!("WeaselVision: Squirrel process not found, sync signal not sent");
            Ok(false)
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("Sync is only supported on macOS and Windows");
    }
}
