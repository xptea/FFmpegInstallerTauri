use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use futures_util::StreamExt;
use tauri::Manager;
use serde::Serialize;
use std::process::Command;

use reqwest;
use zip;

#[derive(Serialize, Clone)]
struct ProgressPayload {
    action: String,
    progress: u8,
}

fn is_ffmpeg_installed() -> bool {
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        output.status.success()
    } else {
        false
    }
}

#[tauri::command]
async fn install_ffmpeg(window: tauri::Window) -> Result<(), String> {
    if is_ffmpeg_installed() {
        window.emit("install_progress", ProgressPayload {
            action: "FFmpeg already installed".to_string(),
            progress: 100,
        }).map_err(|e| e.to_string())?;
    } else {
        // Existing FFmpeg installation code
        let ffmpeg_url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
        let app_data_dir = window.app_handle().path_resolver().app_data_dir().unwrap();
        let ffmpeg_dir = app_data_dir.join("ffmpeg");
        let zip_path = ffmpeg_dir.join("ffmpeg.zip");

        fs::create_dir_all(&ffmpeg_dir).map_err(|e| e.to_string())?;

        // Download FFmpeg
        let response = reqwest::get(ffmpeg_url).await.map_err(|e| e.to_string())?;
        let total_size = response.content_length().unwrap_or(0);
        let mut file = fs::File::create(&zip_path).map_err(|e| e.to_string())?;
        let mut downloaded = 0;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;
            let progress = (downloaded as f64 / total_size as f64 * 100.0) as u8;
            window.emit("install_progress", ProgressPayload {
                action: "Downloading FFmpeg".to_string(),
                progress,
            }).map_err(|e| e.to_string())?;
        }

        // Extract FFmpeg
        let file = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let total_files = archive.len();
        for i in 0..total_files {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => ffmpeg_dir.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }

            let progress = ((i as f64 + 1.0) / total_files as f64 * 100.0) as u8;
            window.emit("install_progress", ProgressPayload {
                action: "Extracting FFmpeg".to_string(),
                progress,
            }).map_err(|e| e.to_string())?;
        }

        // Update PATH
        window.emit("install_progress", ProgressPayload {
            action: "Updating system PATH".to_string(),
            progress: 100,
        }).map_err(|e| e.to_string())?;
        let mut path = env::var("PATH").unwrap_or_default();
        path.push_str(&format!(";{}", ffmpeg_dir.to_str().unwrap()));
        env::set_var("PATH", path);

        fs::remove_file(zip_path).map_err(|e| e.to_string())?;
    }

    // Download SkibidiSlicer
    window.emit("install_progress", ProgressPayload {
        action: "Downloading SkibidiSlicer".to_string(),
        progress: 0,
    }).map_err(|e| e.to_string())?;

    let skibidi_url = "https://github.com/xptea/installskibidi/releases/latest/download/Skibidy.Slicer_0.1.0_x64-setup.zip";
    let client = reqwest::Client::new();
    let response = client.get(skibidi_url)
        .header(reqwest::header::USER_AGENT, "SkibidiSlicerInstaller/1.0")
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err("SkibidiSlicer download URL is invalid or the file has been moved. Please check for updates.".to_string());
    } else if !response.status().is_success() {
        return Err(format!("Failed to download SkibidiSlicer: HTTP status {} - {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown error")));
    }

    let total_size = response.content_length().unwrap_or(0);
    let app_data_dir = window.app_handle().path_resolver().app_data_dir().unwrap();
    let zip_path = app_data_dir.join("SkibidiSlicer.zip");
    let mut file = fs::File::create(&zip_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut downloaded = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Error while downloading file: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Error while writing to file: {}", e))?;
        downloaded += chunk.len() as u64;
        let progress = (downloaded as f64 / total_size as f64 * 100.0) as u8;
        window.emit("install_progress", ProgressPayload {
            action: "Downloading SkibidiSlicer".to_string(),
            progress,
        }).map_err(|e| e.to_string())?;
    }

    // Verify the downloaded file
    if !is_valid_zip(&zip_path) {
        return Err(format!("Downloaded file is not a valid zip archive. File size: {} bytes", fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0)));
    }

    // Extract SkibidiSlicer
    window.emit("install_progress", ProgressPayload {
        action: "Extracting SkibidiSlicer".to_string(),
        progress: 0,
    }).map_err(|e| e.to_string())?;

    let file = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let extract_dir = app_data_dir.join("SkibidiSlicer");
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    let total_files = archive.len();
    for i in 0..total_files {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }

        let progress = ((i + 1) as f64 / total_files as f64 * 100.0) as u8;
        window.emit("install_progress", ProgressPayload {
            action: "Extracting SkibidiSlicer".to_string(),
            progress,
        }).map_err(|e| e.to_string())?;
    }

    // Find and run the SkibidiSlicer executable
    let exe_path = find_exe_in_dir(&extract_dir).ok_or("SkibidiSlicer executable not found")?;
    
    Command::new(&exe_path)
        .spawn()
        .map_err(|e| e.to_string())?;

    // Close the installer
    window.close().map_err(|e| e.to_string())?;

    Ok(())
}

fn find_exe_in_dir(dir: &Path) -> Option<std::path::PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("exe") {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn is_valid_zip(path: &Path) -> bool {
    if let Ok(file) = fs::File::open(path) {
        if let Ok(archive) = zip::ZipArchive::new(file) {
            return archive.len() > 0;
        }
    }
    false
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
        .invoke_handler(tauri::generate_handler![install_ffmpeg])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}