use anyhow::Result;

pub fn deploy() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Use pkill for atomic process matching and signaling (avoids TOCTOU race condition)
        // -f matches against full process name, -HUP sends reload signal
        // Exit code 0 = signal sent, 1 = no matching process (both are acceptable)
        match std::process::Command::new("pkill")
            .args(["-HUP", "-f", "Squirrel"])
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
        let deployer_paths = [
            dirs::program_files().map(|p| p.join("Rime/WeaselDeployer.exe")),
            dirs::program_files_x86().map(|p| p.join("Rime/WeaselDeployer.exe")),
        ];

        for path_opt in deployer_paths {
            if let Some(path) = path_opt {
                if path.exists() {
                    let output = std::process::Command::new(&path)
                        .arg("/deploy")
                        .output()?;
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        anyhow::bail!("WeaselDeployer /deploy failed: {}", stderr);
                    }
                    return Ok(());
                }
            }
        }

        anyhow::bail!("WeaselDeployer.exe not found. Is Weasel installed?");
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
        let deployer_paths = [
            dirs::program_files().map(|p| p.join("Rime/WeaselDeployer.exe")),
            dirs::program_files_x86().map(|p| p.join("Rime/WeaselDeployer.exe")),
        ];

        for path_opt in deployer_paths {
            if let Some(path) = path_opt {
                if path.exists() {
                    let output = std::process::Command::new(&path)
                        .arg("/sync")
                        .output()?;
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        anyhow::bail!("WeaselDeployer /sync failed: {}", stderr);
                    }
                    return Ok(true);
                }
            }
        }
        anyhow::bail!("WeaselDeployer.exe not found");
    }

    #[cfg(target_os = "macos")]
    {
        // Check if Squirrel is running before sending signal
        let is_running = std::process::Command::new("pkill")
            .args(["-0", "-f", "Squirrel"])
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
