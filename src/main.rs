mod interop;

use bindings::windows::{
    system::{DispatcherQueue, DispatcherQueueController, DispatcherQueueHandler},
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
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Once,
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
        RoInitialize(RO_INIT_TYPE::RO_INIT_MULTITHREADED).ok()?;
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
    println!("pressed!");
    Ok(())
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
