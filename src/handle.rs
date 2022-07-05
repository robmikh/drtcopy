use windows::{
    core::{Error, Result},
    Win32::Foundation::HANDLE,
};

pub trait CheckHandle: Sized {
    fn ok(self) -> Result<Self>;
}

impl CheckHandle for HANDLE {
    fn ok(self) -> Result<Self> {
        if self.0 == 0 {
            Err(Error::from_win32())
        } else {
            Ok(self)
        }
    }
}
