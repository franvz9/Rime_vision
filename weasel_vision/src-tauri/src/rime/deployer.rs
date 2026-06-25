use anyhow::Result;

pub fn deploy() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"
        tell application "System Events"
            set procList to do shell script "ps -A | grep -i squirrel | grep -v grep | head -1"
            if procList is not "" then
                set pid to do shell script "ps -A | grep -i squirrel | grep -v grep | awk '{print $1}' | head -1"
                do shell script "kill -HUP " & pid
            end if
        end tell
        "#;
        let output = std::process::Command::new("osascript")
            .args(["-e", script])
            .output();

        match output {
            Ok(o) if o.status.success() => return Ok(()),
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if !stderr.is_empty() {
                    eprintln!("WeaselVision: deploy signal failed: {}", stderr);
                }
            }
            Err(e) => {
                eprintln!("WeaselVision: failed to run osascript: {}", e);
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
                    std::process::Command::new(&path)
                        .arg("/deploy")
                        .output()?;
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

pub fn sync() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let deployer_paths = [
            dirs::program_files().map(|p| p.join("Rime/WeaselDeployer.exe")),
            dirs::program_files_x86().map(|p| p.join("Rime/WeaselDeployer.exe")),
        ];

        for path_opt in deployer_paths {
            if let Some(path) = path_opt {
                if path.exists() {
                    std::process::Command::new(&path)
                        .arg("/sync")
                        .output()?;
                    return Ok(());
                }
            }
        }
        anyhow::bail!("WeaselDeployer.exe not found");
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: sync via notification (same as deploy)
        deploy()?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        anyhow::bail!("Sync is only supported on macOS and Windows");
    }

    Ok(())
}
