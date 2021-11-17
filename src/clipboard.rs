use windows::{
    core::{Handle, Result},
    Win32::{
        Foundation::{HANDLE, HWND},
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
            SystemServices::CF_UNICODETEXT,
        },
    },
};

pub fn set_clipboard_text(text: &str) -> Result<()> {
    unsafe {
        let _clipboard = Clipboard::new()?;
        EmptyClipboard().ok()?;
        let clipboard_data = allocate_text(text)?;
        let _ = SetClipboardData(CF_UNICODETEXT.0, HANDLE(clipboard_data.0)).ok();
        std::mem::forget(clipboard_data);
    }
    Ok(())
}

struct Clipboard;

impl Clipboard {
    fn new() -> Result<Self> {
        unsafe { OpenClipboard(HWND(0)).ok()? };
        Ok(Self {})
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe { CloseClipboard().ok().unwrap() }
    }
}

struct GlobalAllocation(pub isize);

impl GlobalAllocation {
    pub fn new(size: usize) -> Result<Self> {
        let data = unsafe { GlobalAlloc(GMEM_MOVEABLE, size) };
        let temp = HANDLE(data);
        temp.ok()?;
        Ok(Self(data))
    }

    pub fn lock(&self) -> Result<GlobalAllocationLock> {
        let ptr = unsafe { GlobalLock(self.0) };
        let temp = HANDLE(ptr as isize);
        temp.ok()?;
        Ok(GlobalAllocationLock { ptr, source: &self })
    }
}

impl Drop for GlobalAllocation {
    fn drop(&mut self) {
        unsafe {
            GlobalFree(self.0);
        }
    }
}

struct GlobalAllocationLock<'a> {
    ptr: *mut std::ffi::c_void,
    source: &'a GlobalAllocation,
}

impl<'a> GlobalAllocationLock<'a> {
    pub fn as_mut_ptr<T>(&self) -> *mut T {
        unsafe { std::mem::transmute(self.ptr) }
    }
}

impl<'a> Drop for GlobalAllocationLock<'a> {
    fn drop(&mut self) {
        unsafe {
            GlobalUnlock(self.source.0);
        }
    }
}

fn allocate_text(text: &str) -> Result<GlobalAllocation> {
    let mut wide_text: Vec<_> = text.encode_utf16().collect();
    wide_text.push(0);
    let data = GlobalAllocation::new(wide_text.len() * std::mem::size_of::<u16>())?;
    unsafe {
        let lock = data.lock()?;
        let clipboard_slice = std::slice::from_raw_parts_mut(lock.as_mut_ptr(), wide_text.len());
        clipboard_slice.copy_from_slice(&wide_text);
    }
    Ok(data)
}
