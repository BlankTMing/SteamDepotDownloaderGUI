// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, io};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::terminal::Terminal;

mod steam;
mod depotdownloader;
mod terminal;

static DEPOTDOWNLOADER_VERSION: &str = "2.7.1";
//TODO: arm
static DEPOTDOWNLOADER_LINUX_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_2.7.1/DepotDownloader-linux-x64.zip";
static DEPOTDOWNLOADER_WIN_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_2.7.1/DepotDownloader-windows-x64.zip";
static DEPOTDOWNLOADER_MAC_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_2.7.1/DepotDownloader-macos-x64.zip";


// We create this variable now, and quickly populate it in preload_vectum(). Then we later access the data in start_download()
static TERMINAL: OnceLock<Vec<Terminal>> = OnceLock::new();

/// This function is called every time the app is reloaded/started. It quickly populates the [`TERMINAL`] variable with a working terminal.
#[tauri::command]
async fn preload_vectum(app: AppHandle) {
    // Only fill this variable once.
    if TERMINAL.get().is_none() { TERMINAL.set(terminal::get_installed_terminals(true).await).expect("Failed to set available terminals") }

    // Send the default terminal name to the frontend.
    app.emit("default-terminal", Terminal::pretty_name(&TERMINAL.get().unwrap()[0])).unwrap();
}

#[tauri::command]
async fn start_download(steam_download: steam::SteamDownload) {
    let default_terminal = TERMINAL.get().unwrap();
    let working_dir = std::env::current_dir().unwrap();

    let terminal_to_use = if steam_download.options().terminal().is_none() { default_terminal.first().unwrap() } else { &Terminal::from_index(&steam_download.options().terminal().unwrap()).unwrap() };

    println!("-------------------------DEBUG INFO------------------------");
    println!("received these values from frontend:");
    println!("\t- Username: {}", steam_download.username().as_ref().unwrap_or(&String::from("Not provided")));
    // println!("\t- Password: {}", steam_download.password().as_ref().unwrap_or(&String::from("Not provided"))); Don't log in prod lol
    println!("\t- App ID: {}", steam_download.app_id());
    println!("\t- Depot ID: {}", steam_download.depot_id());
    println!("\t- Manifest ID: {}", steam_download.manifest_id());
    println!("\t- Output Path: {}", steam_download.output_path());
    println!("------------------------DEBUG INFORMATION-----------------");
    println!("\t- Default terminal: {}", Terminal::pretty_name(&default_terminal[0]));
    println!("\t- Terminal command: {:?}", terminal_to_use.create_command(&steam_download));
    println!("\t- Working directory: {}", working_dir.display());
    println!("----------------------------------------------------------");

    terminal_to_use.create_command(&steam_download).spawn().ok();
}


/// Downloads the DepotDownloader zip file from the internet based on the OS.
#[tauri::command]
async fn download_depotdownloader() {
    let url = match get_os() {
        "linux"  => DEPOTDOWNLOADER_LINUX_URL,
        "macos" => DEPOTDOWNLOADER_MAC_URL,
        "windows" => DEPOTDOWNLOADER_WIN_URL,
        _ => DEPOTDOWNLOADER_LINUX_URL,
    };
    
    // Where we store the DepotDownloader zip.
    let zip_filename = format!("DepotDownloader-v{}-{}.zip", DEPOTDOWNLOADER_VERSION, env::consts::OS);
    let depotdownloader_zip = Path::new(&zip_filename);

    println!("Downloading DepotDownloader for {} to .{}{}", env::consts::OS, std::path::MAIN_SEPARATOR, depotdownloader_zip.display());

    match depotdownloader::download_file(url, depotdownloader_zip).await {
        Err(e) => {
            if e.kind() == io::ErrorKind::AlreadyExists {
                println!("DepotDownloader already exists. Skipping download.");
                return;
            }
            
            println!("Failed to download DepotDownloader: {}", e);
            return;
        },
        _ => {}
    }
    
    println!("Succesfully downloaded DepotDownloader from {}", url);

    depotdownloader::unzip(depotdownloader_zip).unwrap();
    println!("Succesfully extracted DepotDownloader zip.");
}

/// Checks internet connectivity using Google
#[tauri::command]
async fn internet_connection() -> bool {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();

    client.get("https://connectivitycheck.android.com/generate_204").send().await.is_ok()
}


#[tauri::command]
async fn get_all_terminals(app: AppHandle) {
    let terminals = terminal::get_installed_terminals(false).await;

    terminals.iter().for_each(|terminal| {
        println!("Terminal #{} ({}) is installed!", terminal.index().unwrap(), terminal.pretty_name());
        
        // Sends: (terminal index aligned with dropdown; total terminals)
        app.emit("working-terminal", (terminal.index(), Terminal::total())).unwrap();
    });
}

pub fn get_os() -> &'static str {
    match env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        _ => "unknown",
    }
}

fn main() {
    println!();
    tauri::Builder::default().plugin(tauri_plugin_dialog::init()).plugin(tauri_plugin_shell::init()).invoke_handler(tauri::generate_handler![
            start_download,
            download_depotdownloader,
            internet_connection,
            preload_vectum,
            get_all_terminals
        ]).run(tauri::generate_context!()).expect("error while running tauri application");
}
