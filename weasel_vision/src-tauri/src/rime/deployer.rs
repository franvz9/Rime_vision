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
    deployer_paths.push(std::path::PathBuf::from(
        "C:\\Program Files\\Rime\\WeaselDeployer.exe",
    ));
    deployer_paths.push(std::path::PathBuf::from(
        "C:\\Program Files (x86)\\Rime\\WeaselDeployer.exe",
    ));

    // Method 3: Check user's AppData\Local\Programs
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        deployer_paths.push(
            std::path::PathBuf::from(&local_appdata).join("Programs\\Rime\\WeaselDeployer.exe"),
        );
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
        // Squirrel does NOT handle SIGHUP — sending it will kill the process and
        // may cause macOS to unregister it from the input source list.
        // The correct way to trigger a redeploy is to run `Squirrel --reload`,
        // which posts a SquirrelReloadNotification via DistributedNotificationCenter.
        // The running Squirrel process receives the notification and calls deploy().
        let squirrel_bin = "/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel";
        eprintln!("WeaselVision: Triggering Squirrel reload via --reload command...");
        let output = std::process::Command::new(squirrel_bin)
            .arg("--reload")
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute Squirrel --reload: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("WeaselVision: Squirrel --reload error - exit code {:?}, stderr: {}", output.status.code(), stderr);
            anyhow::bail!(
                "Squirrel --reload failed (exit code {:?}): {}",
                output.status.code(),
                stderr
            );
        }
        eprintln!("WeaselVision: Squirrel reload notification sent successfully");
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

        let install_dir = deployer_path.parent().ok_or_else(|| {
            anyhow::anyhow!("Unable to locate Weasel install directory")
        })?;
        let server_path = install_dir.join("WeaselServer.exe");
        if server_path.exists() {
            let _ = std::process::Command::new(&server_path).arg("/q").output();
            std::process::Command::new(&server_path)
                .current_dir(install_dir)
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to restart WeaselServer.exe: {}", e))?;
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("Deploy is only supported on macOS and Windows");
    }

    // deploy 成功后，记录 build_dir 已更新（缓存不可变，供调用方参考）
    let cfg = super::config::RimeConfig::detect();
    let refreshed = super::config::RimeConfig::detect_build_dir(&cfg.user_dir);
    eprintln!("deploy 完成，build_dir 已刷新: {:?}", refreshed);

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
        // Check if Squirrel is running using pgrep (read-only, no signal sent)
        let check = std::process::Command::new("pgrep")
            .args(["-x", "Squirrel"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute pgrep: {}", e))?;

        if !check.status.success() {
            eprintln!("WeaselVision: Squirrel process not found, sync signal not sent");
            return Ok(false);
        }

        // Squirrel does NOT handle SIGHUP — use `Squirrel --sync` instead,
        // which posts a SquirrelSyncNotification via DistributedNotificationCenter.
        let squirrel_bin = "/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel";
        eprintln!("WeaselVision: Triggering Squirrel sync via --sync command...");
        let output = std::process::Command::new(squirrel_bin)
            .arg("--sync")
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to execute Squirrel --sync: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("WeaselVision: Squirrel --sync error - exit code {:?}, stderr: {}", output.status.code(), stderr);
            anyhow::bail!(
                "Squirrel --sync failed (exit code {:?}): {}",
                output.status.code(),
                stderr
            );
        }
        eprintln!("WeaselVision: Squirrel sync notification sent successfully");
        Ok(true)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("Sync is only supported on macOS and Windows");
    }
}
