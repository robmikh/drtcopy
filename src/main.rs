mod hotkey;

use hotkey::HotKey;
use std::fs::{read_dir, File};
use std::path::Path;
use std::time::SystemTime;
use std::{io::Read, path::PathBuf};
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_CONTROL, MOD_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::WM_HOTKEY;
use windows::{
    core::Result,
    ApplicationModel::DataTransfer::{Clipboard, DataPackage, DataPackageOperation},
    Storage::KnownFolders,
    Win32::{
        Foundation::HWND,
        System::WinRT::{RoInitialize, RO_INIT_SINGLETHREADED},
        UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, MSG},
    },
};

fn main() -> Result<()> {
    unsafe {
        RoInitialize(RO_INIT_SINGLETHREADED)?;
    }

    // 0x4A is the J key
    let _hot_key = HotKey::new(MOD_SHIFT | MOD_CONTROL, 0x4A)?;

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
    let documents_folder = KnownFolders::DocumentsLibrary()?;
    let save_folder = documents_folder
        .GetFolderAsync("Warcraft III\\CustomMapData\\DRT1")?
        .get()?;
    let save_path = save_folder.Path()?.to_string();
    let save_path = Path::new(&save_path);
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
    let package = DataPackage::new()?;
    package.SetRequestedOperation(DataPackageOperation::Copy)?;
    package.SetText(code)?;
    Clipboard::SetContent(package)?;
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
    println!("Loading file \"{:#?}\"...", &file_path);
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
