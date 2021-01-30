mod interop;

use bindings::windows::{
    application_model::data_transfer::{Clipboard, DataPackage, DataPackageOperation},
    storage::KnownFolders,
    system::{DispatcherQueue, DispatcherQueueHandler},
    win32::{
        keyboard_and_mouse_input::GetKeyState,
        system_services::{HINSTANCE, LRESULT, VK_LWIN, WH_KEYBOARD_LL, WM_KEYUP},
        windows_and_messaging::{
            CallNextHookEx, DispatchMessageA, GetMessageA, SetWindowsHookExA, UnhookWindowsHookEx,
            HWND, KBDLLHOOKSTRUCT, LPARAM, MSG, WPARAM,
        },
        winrt::{RoInitialize, RO_INIT_TYPE},
    },
};
use interop::create_dispatcher_queue_controller_for_current_thread;
use std::fs::{read_dir, File};
use std::path::Path;
use std::time::SystemTime;
use std::{
    io::Read,
    path::PathBuf,
    sync::Once,
};

static mut MAIN_THREAD_QUEUE: Option<DispatcherQueue> = None;
static INIT: Once = Once::new();

#[repr(transparent)]
struct HHook(isize);

impl Drop for HHook {
    fn drop(&mut self) {
        unsafe {
            UnhookWindowsHookEx(self.0).ok().unwrap();
        }
    }
}

fn main() -> windows::Result<()> {
    unsafe {
        RoInitialize(RO_INIT_TYPE::RO_INIT_SINGLETHREADED).ok()?;
    }

    let controller = create_dispatcher_queue_controller_for_current_thread()?;
    let queue = controller.dispatcher_queue()?;
    INIT.call_once(|| unsafe {
        MAIN_THREAD_QUEUE = Some(queue);
    });

    let hook = unsafe {
        HHook(SetWindowsHookExA(
            WH_KEYBOARD_LL,
            Some(hook_proc),
            HINSTANCE(0),
            0,
        ))
    };
    assert!(hook.0 != 0);

    unsafe {
        let mut message = MSG::default();
        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&mut message);
        }
    }

    Ok(())
}

fn copy_code() -> windows::Result<()> {
    let documents_folder = KnownFolders::documents_library()?;
    let save_folder = documents_folder
        .get_folder_async("Warcraft III\\CustomMapData\\DRT1")?
        .get()?;
    let save_path = save_folder.path()?.to_string();
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
    package.set_requested_operation(DataPackageOperation::Copy)?;
    package.set_text(code)?;
    Clipboard::set_content(package)?;
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

extern "system" fn hook_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    unsafe {
        if code == 0 {
            // w_param holds the identifier of the keyboard message
            if w_param.0 as i32 == WM_KEYUP {
                // l_param holds a pointer to a KBDLLHOOKSTRUCT struct
                let keyboard_info: *mut KBDLLHOOKSTRUCT = std::mem::transmute(l_param);
                let keyboard_info = keyboard_info.as_ref().unwrap();

                // 0x4A is the J key
                if keyboard_info.vk_code == 0x4A {
                    // Check to see if the windows key is also down
                    let key_state = GetKeyState(VK_LWIN);
                    if key_state != 0 {
                        // Signal the main thread to find and copy the new save code
                        let main_thread_queue = MAIN_THREAD_QUEUE.clone().unwrap();
                        main_thread_queue
                            .try_enqueue(DispatcherQueueHandler::new(
                                move || -> windows::Result<()> {
                                    copy_code()?;
                                    Ok(())
                                },
                            ))
                            .unwrap();
                    }
                }
            }
        }

        return CallNextHookEx(0, code, w_param, l_param);
    }
}
