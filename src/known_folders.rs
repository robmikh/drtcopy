use std::{
    ffi::OsString,
    os::windows::prelude::OsStringExt,
    path::{Path, PathBuf},
};
use windows::{
    core::{Result, PWSTR},
    Win32::{
        System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER},
        UI::Shell::{FOLDERID_Documents, IKnownFolderManager, KnownFolderManager, KF_FLAG_DEFAULT},
    },
};

pub fn get_documents_folder() -> Result<PathBuf> {
    unsafe {
        let folder_manager: IKnownFolderManager =
            CoCreateInstance(&KnownFolderManager, None, CLSCTX_INPROC_SERVER)?;
        let folder = folder_manager.GetFolder(&FOLDERID_Documents)?;
        let path_string = folder.GetPath(KF_FLAG_DEFAULT.0 as u32)?;
        let path = pwstr_to_path(&path_string);
        CoTaskMemFree(path_string.0 as *const _);
        Ok(path)
    }
}

fn pwstr_to_path(source: &PWSTR) -> PathBuf {
    unsafe {
        let len = pwstr_len(source);
        let slice = std::slice::from_raw_parts(source.0, len);
        let str = OsString::from_wide(slice);
        Path::new(&str).to_owned()
    }
}

fn pwstr_len(source: &PWSTR) -> usize {
    unsafe {
        let mut current = source.0;
        let mut count = 0;
        while *current != 0 {
            count += 1;
            current = current.add(1);
        }
        count
    }
}
