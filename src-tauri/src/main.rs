#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use futures_util::StreamExt;
use reqwest;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::{Manager, Window};
use winreg::enums::*;
use winreg::RegKey;
use zip;

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    message: String,
    percent: f32,
}

fn send_progress(window: &Window, message: &str, percent: f32) -> Result<(), String> {
    window
        .emit(
            "progress",
            ProgressPayload {
                message: message.to_string(),
                percent,
            },
        )
        .map_err(|e| e.to_string())
}

fn add_to_user_path(new_path: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", winreg::enums::KEY_READ | winreg::enums::KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    let current_path: String = env.get_value("PATH").unwrap_or_default();

    if current_path.contains(new_path) {
        return Ok(());
    }

    let new_full_path = format!("{};{}", current_path, new_path);

    env.set_value("PATH", &new_full_path)
        .map_err(|e| format!("Failed to update PATH in registry: {}", e))?;

    Ok(())
}



#[tauri::command]
async fn install_ffmpeg_and_skibidi(window: Window) -> Result<(), String> {
    // Step 1: Check if FFmpeg is installed
    let ffmpeg_check = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output();

    match ffmpeg_check {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("ffmpeg version") {
                    send_progress(&window, "FFmpeg is already installed", 100.0)?;
                    return Ok(());
                } else {
                    send_progress(&window, "FFmpeg not found, installing...", 0.0)?;
                }
            } else {
                send_progress(&window, "FFmpeg not found, installing...", 0.0)?;
            }
        }
        Err(_) => {
            send_progress(&window, "FFmpeg not found, installing...", 0.0)?;
        }
    }

    // Step 2: Download and Install FFmpeg
    let ffmpeg_url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
    let ffmpeg_install_path = PathBuf::from(r"C:\ffmpeg");

    if !ffmpeg_install_path.exists() {
        fs::create_dir_all(&ffmpeg_install_path).map_err(|e| e.to_string())?;
    }

    send_progress(&window, "Downloading FFmpeg...", 10.0)?;

    let response = reqwest::get(ffmpeg_url).await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    let zip_file_path = ffmpeg_install_path.join("ffmpeg.zip");
    let mut file = fs::File::create(&zip_file_path).map_err(|e| e.to_string())?;

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        let percent = if total_size > 0 {
            (downloaded as f32 / total_size as f32) * 90.0
        } else {
            50.0 // fallback if content length is unknown
        };

        send_progress(&window, "Downloading FFmpeg...", 10.0 + percent)?;
    }

    file.sync_all().map_err(|e| e.to_string())?;

    if downloaded == 0 {
        return Err("Download failed or resulted in an empty file.".into());
    }

    send_progress(&window, "Extracting FFmpeg...", 95.0)?;

    let mut archive =
        match zip::ZipArchive::new(fs::File::open(&zip_file_path).map_err(|e| e.to_string())?) {
            Ok(archive) => archive,
            Err(_) => {
                return Err("Invalid zip archive or corrupted download.".into());
            }
        };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let out_path =
            ffmpeg_install_path.join(file.enclosed_name().ok_or("Invalid file path in archive")?);

        if (*file.name()).ends_with('/') {
            if !out_path.exists() {
                fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            }
        } else {
            let mut outfile = fs::File::create(&out_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    send_progress(&window, "FFmpeg installation completed!", 100.0)?;

    let ffmpeg_bin_path = r"C:\ffmpeg\ffmpeg-master-latest-win64-gpl\bin";
    add_to_user_path(ffmpeg_bin_path)?;

    // Step 3: Download and Install Skibidi Slicer
    let skibidi_url = "https://github.com/xptea/installskibidi/releases/latest/download/Skibidy.Slicer_0.1.0_x64-setup.zip";
    let skibidi_install_path = PathBuf::from(r"C:\SkibidySlicer");

    if !skibidi_install_path.exists() {
        fs::create_dir_all(&skibidi_install_path).map_err(|e| e.to_string())?;
    }

    send_progress(&window, "Downloading Skibidi Slicer...", 0.0)?;

    let response = match reqwest::get(skibidi_url).await {
        Ok(res) => res,
        Err(e) => return Err(format!("Failed to download Skibidi Slicer: {}", e)),
    };
    
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    
    let zip_file_path = skibidi_install_path.join("SkibidySlicer.zip");
    let mut file = fs::File::create(&zip_file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| format!("Failed to write to file: {}", e))?;
        downloaded += chunk.len() as u64;
    
        let percent = if total_size > 0 {
            (downloaded as f32 / total_size as f32) * 100.0
        } else {
            50.0
        };
    
        send_progress(&window, &format!("Downloading Skibidy Slicer: {}%", percent), percent)?;
    }
    
    file.sync_all().map_err(|e| format!("Failed to sync file: {}", e))?;
    send_progress(&window, "Extracting Skibidy Slicer...", 100.0)?;
    
    let mut archive = match zip::ZipArchive::new(fs::File::open(&zip_file_path).map_err(|e| format!("Failed to open zip file: {}", e))?) {
        Ok(archive) => archive,
        Err(e) => return Err(format!("Invalid zip archive or corrupted download: {}", e)),
    };
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Failed to access file in archive: {}", e))?;
        let out_path = skibidi_install_path.join(file.enclosed_name().ok_or("Invalid file path in archive")?);
        send_progress(&window, &format!("Extracting: {:?}", out_path), 100.0)?;
    
        if (*file.name()).ends_with('/') {
            if !out_path.exists() {
                fs::create_dir_all(&out_path).map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        } else {
            let mut outfile = fs::File::create(&out_path).map_err(|e| format!("Failed to create output file: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    send_progress(&window, "Skibidi Slicer installation completed!", 100.0)?;

    // Step 4: Launch the Skibidi Slicer installer
    let installer_path = skibidi_install_path.join("Skibidy Slicer_0.1.0_x64-setup.exe"); // Update the path to the correct installer name
    if installer_path.exists() {
        std::process::Command::new(installer_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Skibidi Slicer installer: {}", e))?;
    
        send_progress(&window, "Skibidi Slicer installer launched successfully.", 100.0)?;
    } else {
        return Err(format!("Installer file not found at: {:?}", installer_path));
    }
    
    // Close the application after launching the installer
    send_progress(&window, "Closing the app...", 100.0)?;
    std::process::exit(0);
    
}


fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![install_ffmpeg_and_skibidi])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}