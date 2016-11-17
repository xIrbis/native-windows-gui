/*
    Copyright (C) 2016  Gabriel Dubé

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

pub mod helper;
mod sysproc;

pub use ::controls::base::helper::*;
use ::controls::base::sysproc::{sub_wndproc, NWG_DESTROY_NOTICE};
use constants::Error;

use std::ptr;
use std::mem;
use std::hash::Hash;

use winapi::{HWND, ACTCTXW, ULONG, ULONG_PTR, MAX_PATH, LPARAM, BOOL,
  WS_VISIBLE, WS_CHILD, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_CAPTION, WS_SYSMENU,
  WS_MINIMIZEBOX, WS_MAXIMIZEBOX, WS_EX_COMPOSITED, GWLP_USERDATA};

use user32::{CreateWindowExW, DestroyWindow, EnumChildWindows};
use kernel32::{GetModuleHandleW, ActivateActCtx, CreateActCtxW, GetSystemDirectoryW};

use comctl32::{SetWindowSubclass};

pub struct WindowBase<ID: Eq+Hash+Clone> {
    pub text: String,
    pub size: (u32, u32),
    pub position: (i32, i32),
    pub visible: bool,
    pub resizable: bool,
    pub extra_style: u32,
    pub class: String,
    pub parent: Option<ID>
}

/**
    Create a new window. The window details is determined by the base 
    parameters passed to the function.

    If successful, return an handle to the new window.
*/
pub unsafe fn create_base<ID: Eq+Clone+Hash>(ui: &mut ::Ui<ID>, base: WindowBase<ID>) -> Result<HWND, ()> {
    let hmod = GetModuleHandleW(ptr::null());

    // Resolve the parent if provided, else return an empty handle
    let parent: HWND = match base.parent {
        Some(id) => {
            let controls: &mut ::ControlCollection<ID> = &mut *ui.controls;
            match controls.get(&id) {
                Some(&(h,_)) => h,
                None => { return Err(()); }
            }
        },
        None => ptr::null_mut()
    };

    let class_name = to_utf16(base.class);
    let window_name = to_utf16(base.text);

    // Eval the window flags
    let mut flags = 0;
    if base.visible { flags |= WS_VISIBLE; }
    if !parent.is_null() { flags |= WS_CHILD; }
    if parent.is_null() { 
        if base.resizable { flags |= WS_OVERLAPPEDWINDOW; }
        else { flags |= WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX}
    }

    flags |= base.extra_style;

    let hwnd = CreateWindowExW(
        WS_EX_COMPOSITED, class_name.as_ptr(), window_name.as_ptr(),
        flags,
        base.position.0, base.position.1,
        base.size.0 as i32, base.size.1 as i32,
        parent,
        ptr::null_mut(),
        hmod,
        ptr::null_mut()
    );

    if hwnd.is_null() {
        Err(())
    } else {
        if flags & WS_OVERLAPPEDWINDOW != 0 {
            fix_overlapped_window_size(hwnd, base.size);
        }
        
        SetWindowSubclass(hwnd, Some(sub_wndproc::<ID>), 1, 0);
        Ok(hwnd)
    }
}

/**
    hackish way to enable visual style without a manifest
*/
pub unsafe fn enable_visual_styles() {
    use constants::{ACTCTX_FLAG_RESOURCE_NAME_VALID, ACTCTX_FLAG_SET_PROCESS_DEFAULT, ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID};

    let mut sys_dir: Vec<u16> = Vec::with_capacity(MAX_PATH);
    sys_dir.set_len(MAX_PATH);
    GetSystemDirectoryW(sys_dir.as_mut_ptr(), MAX_PATH as u32);

    let mut source = to_utf16("shell32.dll".to_string());

    let mut activation_cookie: ULONG_PTR = 0;
    let mut act_ctx = ACTCTXW {
        cbSize: mem::size_of::<ACTCTXW> as ULONG,
        dwFlags: ACTCTX_FLAG_RESOURCE_NAME_VALID | ACTCTX_FLAG_SET_PROCESS_DEFAULT | ACTCTX_FLAG_ASSEMBLY_DIRECTORY_VALID,
        lpSource: source.as_mut_ptr(),
        wProcessorArchitecture: 0,
        wLangId: 0,
        lpAssemblyDirectory: sys_dir.as_mut_ptr(),
        lpResourceName: mem::transmute(124usize), // ID_MANIFEST
        lpApplicationName: ptr::null_mut(),
        hModule: ptr::null_mut()
    };

    let handle = CreateActCtxW(&mut act_ctx);
    ActivateActCtx(handle, &mut activation_cookie);
}

////
//// Window data helper
////

/**
    Store data in a window
*/
pub unsafe fn set_handle_data<T>(handle: HWND, data: T) {
    let data_raw = Box::into_raw(Box::new(data));
    set_window_long(handle, GWLP_USERDATA, mem::transmute(data_raw));
}

/**
    Store data in a window using an offset. To use to store custom widget private data.
*/
pub unsafe fn set_handle_data_off<T>(handle: HWND, data: T, offset: usize) {
    let data_raw = Box::into_raw(Box::new(data));
    set_window_long(handle, (offset*mem::size_of::<usize>()) as i32, mem::transmute(data_raw));
}

/**
    Retrieve data in a window
*/
pub unsafe fn get_handle_data<'a, T>(handle: HWND) -> Option<&'a mut T> {
    let data_ptr = get_window_long(handle, GWLP_USERDATA);
    if data_ptr != 0 {
        let data: *mut T = mem::transmute(data_ptr);
        Some(&mut *data)
    } else {
        None
    }
}

/**
    Retrieve data in a window using an offset. To use to retrieve custom widget private data.
*/
pub unsafe fn get_handle_data_off<'a, T>(handle: HWND, offset: usize) -> Option<&'a mut T> {
    let data_ptr = get_window_long(handle, (offset*mem::size_of::<usize>()) as i32);
    if data_ptr != 0 {
        let data: *mut T = mem::transmute(data_ptr);
        Some(&mut *data)
    } else {
        None
    }
}

/**
    Remove and free data from a window
*/
pub unsafe fn free_handle_data<T>(handle: HWND) {
    let data_ptr = get_window_long(handle, GWLP_USERDATA);
    let data: *mut T = mem::transmute(data_ptr);
    Box::from_raw(data);

    set_window_long(handle, GWLP_USERDATA, mem::transmute(ptr::null_mut::<()>()));
}

/**
    Remove and free data from a window using an offset
*/
pub unsafe fn free_handle_data_off<T>(handle: HWND, offset: usize) {
    let data_ptr = get_window_long(handle, (offset*mem::size_of::<usize>()) as i32 );
    let data: *mut T = mem::transmute(data_ptr);
    Box::from_raw(data);

    set_window_long(handle, (offset*mem::size_of::<usize>()) as i32, mem::transmute(ptr::null_mut::<()>()));
}

/// Proc used to discover a window children
unsafe extern "system" fn notice_child_destroy(handle: HWND, param: LPARAM) -> BOOL {
     send_message(handle, NWG_DESTROY_NOTICE, 0, 0);
     1
}

unsafe extern "system" fn free_child_data<ID: Eq+Hash+Clone>(handle: HWND, param: LPARAM) -> BOOL {
     send_message(handle, NWG_DESTROY_NOTICE, 0, 0);
     free_handle_data::<::WindowData<ID>>(handle);
     1
}

/**
    Recursively destroy the handle and all its children and free any data attached.
    This is called by Ui.remove_control. Does NOT remove the ID from the control collection!
*/
pub unsafe fn free_handle<ID: Eq+Clone+Hash >(handle: HWND) {
    let data_raw: *mut ::WindowData<ID> = mem::transmute(get_window_long(handle, GWLP_USERDATA));
    if !data_raw.is_null() {
        // Noticing
        send_message(handle, NWG_DESTROY_NOTICE, 0, 0);
        EnumChildWindows(handle, Some(notice_child_destroy), 0);

        // Freeing
        EnumChildWindows(handle, Some(free_child_data::<ID>), 0);
        DestroyWindow(handle);
        Box::from_raw(data_raw);
        set_window_long(handle, GWLP_USERDATA, mem::transmute(ptr::null_mut::<()>()));
    }
    
}

/**
    Remove a control from the ui as if Ui.remove_control was called. Used by custom widgets (ex: Window),
    when an event must trigger the control destruction.
*/
pub unsafe fn destroy_control<ID: Eq+Clone+Hash>(handle: HWND) -> Result<Vec<ID>, Error> {
    let data_raw: *mut ::WindowData<ID> = mem::transmute(get_window_long(handle, GWLP_USERDATA));
    if !data_raw.is_null() {
        let data: &mut ::WindowData<ID> = &mut *data_raw;
        let mut ui = ::Ui{controls: data.controls};
        let op = ui.remove_control(data.id.clone());
        mem::forget(ui);
        op
    } else {
        Err(Error::NO_UI)
    }
}