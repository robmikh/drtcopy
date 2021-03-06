fn main() {
    windows::build!(
        windows::win32::windows_and_messaging::{
            HWND,
            UnhookWindowsHookEx,
            SetWindowsHookExA,
            WPARAM,
            LPARAM,
            CallNextHookEx,
            KBDLLHOOKSTRUCT,
            DispatchMessageA,
            GetMessageA,
            MSG,
        },
        windows::win32::system_services::{
            WH_KEYBOARD_LL,
            HINSTANCE,
            WM_KEYUP,
            LRESULT,
            VK_LWIN,
            CreateDispatcherQueueController,
        },
        windows::win32::keyboard_and_mouse_input::{
            GetKeyState,
        },
        windows::win32::winrt::{
            RoInitialize,
            RO_INIT_TYPE,
        },
        windows::application_model::data_transfer::{
            DataPackage, DataPackageOperation, Clipboard,
        },
        windows::storage::{
            StorageFolder, StorageFile, KnownFolders,
        },
        windows::system::{
            DispatcherQueueController,
            DispatcherQueue,
            DispatcherQueueHandler,
        },
    );
}
