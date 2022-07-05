mod clipboard;
mod handle;
mod hotkey;
mod known_folders;

use hotkey::HotKey;
use std::fs::{read_dir, File};
use std::path::Path;
use std::time::SystemTime;
use std::{io::Read, path::PathBuf};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_CONTROL, MOD_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::WM_HOTKEY;
use windows::{
    core::Result,
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG},
    },
};

use crate::clipboard::set_clipboard_text;
use crate::known_folders::get_documents_folder;

fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED)?;
    }

    // 0x4A is the J key
    let _hot_key = HotKey::new(MOD_SHIFT | MOD_CONTROL, 0x4A)?;

    println!("Press SHIFT+CTRL+J to copy your most recent DRT code to the clipboard...");
    unsafe {
        let mut message = MSG::default();
        while GetMessageW(&mut message, HWND(0), 0, 0).into() {
            if message.message == WM_HOTKEY {
                copy_code()?;
            }
            DispatchMessageW(&mut message);
        }
    }

    Ok(())
}

fn copy_code() -> Result<()> {
    let mut save_path = get_documents_folder()?;
    save_path.push("Warcraft III\\CustomMapData\\DRT1");
    if !save_path.exists() {
        println!(
            "Save path \"{}\" does not exist!",
            save_path.to_string_lossy()
        );
        return Ok(());
    }
    let code = find_code(&save_path).unwrap();
    println!("Code found: {}", &code);

    // copy the code to the clipboard
    set_clipboard_text(&code)?;
    println!("Code copied to clipboard!");

    Ok(())
}

fn find_newest_file(path: &Path) -> std::io::Result<PathBuf> {
    let mut result = None;
    let mut newest = SystemTime::UNIX_EPOCH;
    for entry in read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_file() {
            let metadata = entry.metadata()?;
            let modified = metadata.modified()?;
            if modified > newest {
                newest = modified;
                result = Some(entry);
            }
        }
    }
    Ok(result.unwrap().path())
}

fn find_code(save_path: &Path) -> std::io::Result<String> {
    let file_path = find_newest_file(save_path)?;
    println!("Loading file \"{}\"...", file_path.display());
    let contents = {
        let mut contents = String::new();
        let mut file = File::open(file_path)?;
        file.read_to_string(&mut contents)?;
        contents
    };

    let load_index = contents.find("-load").unwrap();
    let temp = &contents[load_index..];
    let quote_index = temp.find(" \"").unwrap();
    let code = &temp[..quote_index];

    Ok(code.to_string())
}
