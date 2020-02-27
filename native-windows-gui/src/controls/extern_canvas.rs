use winapi::um::winuser::{WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_DISABLED, WS_MAXIMIZE, WS_MINIMIZE, WS_CAPTION,
WS_MINIMIZEBOX, WS_MAXIMIZEBOX, WS_SYSMENU, WS_THICKFRAME, WS_CLIPCHILDREN, WS_CLIPSIBLINGS };

use crate::win32::window_helper as wh;
use crate::{NwgError, Icon};
use super::{ControlBase, ControlHandle};

const NOT_BOUND: &'static str = "ExternCanvas is not yet bound to a winapi object";
const BAD_HANDLE: &'static str = "INTERNAL ERROR: ExternCanvas handle is not HWND!";


bitflags! {

    /**
        The extern canvas flags. 
        
        Note that the window flags only applies if the the extern canvas is a top level window (it has no parents).
        
        Window flags:
        * MAIN_WINDOW: Combine all the top level system window decoration: A title, a system menu, a resizable frame, and the close, minimize, maximize buttons
        * WINDOW:  A window with a title, a system menu, a close button, and a non resizable border. 
        * MINIMIZE_BOX: Includes a minimize button
        * MAXIMIZE_BOX: Includes a maximize button
        * SYS_MENU: Includes a system menu when the user right click the window header
        * MAXIMIZED: Create the window as maximized
        * MINIMIZED: Create the window as minimized
        * RESIZABLE: Add a resizable border

        General flags:
        * VISIBLE: Show the window right away
    */
    pub struct ExternCanvasFlags: u32 {
        const NONE = 0;
        const MAIN_WINDOW = WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_THICKFRAME | WS_MAXIMIZEBOX;
        const WINDOW = WS_CAPTION | WS_SYSMENU;
        const MINIMIZE_BOX = WS_MINIMIZEBOX;
        const MAXIMIZE_BOX = WS_MAXIMIZEBOX;
        const SYS_MENU = WS_SYSMENU;
        const VISIBLE = WS_VISIBLE;
        const DISABLED = WS_DISABLED;
        const MAXIMIZED = WS_MAXIMIZE;
        const MINIMIZED = WS_MINIMIZE;
        const RESIZABLE = WS_THICKFRAME | WS_MAXIMIZEBOX;
    }
}

/**
    An ExternCanvas is a window/children control that is painted to by an external API (such as OpenGL, Vulkan or DirectX).
    
    When building a ExternCanvas, leaving the parent field empty will create a window-like canvas. If a parent is set, the canvas
    will be a children control (like a button).

    When used as a chidren, ExternCanvas can be used as a way to add highly dynamic controls to a NWG application (ex: a video player).

    Requires the `extern-canvas` feature. 

*/
#[derive(Default)]
pub struct ExternCanvas {
    pub handle: ControlHandle
}

impl ExternCanvas {

    pub fn builder<'a>() -> ExternCanvasBuilder<'a> {
        ExternCanvasBuilder {
            title: "New Canvas",
            size: (500, 500),
            position: (300, 300),
            flags: None,
            icon: None,
            parent: None
        }
    }

    /// Invalidate the whole drawing region. For canvas that are children control, this should be called in the paint event.
    pub fn invalidate(&self) {
        use winapi::um::winuser::InvalidateRect;
        use std::ptr;

        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        unsafe { InvalidateRect(handle, ptr::null(), 1); }
    }

    /// Return the icon of the window
    pub fn icon(&self) -> Option<Icon> {
        use winapi::um::winuser::WM_GETICON;
        use winapi::um::winnt::HANDLE;

        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let handle = wh::send_message(handle, WM_GETICON, 0, 0);
        if handle == 0 {
            None
        } else {
            Some(Icon { handle: handle as HANDLE, owned: false })
        }
    }

    /// Set the icon in the window
    /// - icon: The new icon. If None, the icon is removed
    pub fn set_icon(&self, icon: Option<&Icon>) {
        use winapi::um::winuser::WM_SETICON;
        use std::{mem, ptr};

        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);

        let image_handle = icon.map(|i| i.handle).unwrap_or(ptr::null_mut());
        unsafe {
            wh::send_message(handle, WM_SETICON, 0, mem::transmute(image_handle));
        }
    }

    /// Return true if the control currently has the keyboard focus
    pub fn focus(&self) -> bool {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_focus(handle) }
    }

    /// Set the keyboard focus on the button
    pub fn set_focus(&self) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_focus(handle); }
    }

    /// Return true if the control user can interact with the control, return false otherwise
    pub fn enabled(&self) -> bool {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_enabled(handle) }
    }

    /// Enable or disable the control
    pub fn set_enabled(&self, v: bool) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_enabled(handle, v) }
    }

    /// Return true if the control is visible to the user. Will return true even if the 
    /// control is outside of the parent client view (ex: at the position (10000, 10000))
    pub fn visible(&self) -> bool {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_visibility(handle) }
    }

    /// Show or hide the control to the user
    pub fn set_visible(&self, v: bool) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_visibility(handle, v) }
    }

    /// Return the size of the button in the parent window
    pub fn size(&self) -> (u32, u32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_size(handle) }
    }

    /// Set the size of the button in the parent window
    pub fn set_size(&self, x: u32, y: u32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_size(handle, x, y, true) }
    }

    /// Return the position of the button in the parent window
    pub fn position(&self) -> (i32, i32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_position(handle) }
    }

    /// Set the position of the button in the parent window
    pub fn set_position(&self, x: i32, y: i32) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_position(handle, x, y) }
    }

    /// Return window title
    pub fn text(&self) -> String { 
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::get_window_text(handle) }
    }

    /// Set the window title
    pub fn set_text<'a>(&self, v: &'a str) {
        if self.handle.blank() { panic!(NOT_BOUND); }
        let handle = self.handle.hwnd().expect(BAD_HANDLE);
        unsafe { wh::set_window_text(handle, v) }
    }

    /// Winapi class name used during control creation
    pub fn class_name(&self) -> &'static str {
        "NWG_EXTERN_CANVAS"
    }

    // Winapi base flags used during window creation
    pub fn flags(&self) -> u32 {
        WS_OVERLAPPEDWINDOW | WS_VISIBLE
    }

    /// Winapi flags required by the control
    pub fn forced_flags(&self) -> u32 {
        WS_CLIPCHILDREN | WS_CLIPSIBLINGS 
    }
}


pub struct ExternCanvasBuilder<'a> {
    title: &'a str,
    size: (i32, i32),
    position: (i32, i32),
    flags: Option<ExternCanvasFlags>,
    icon: Option<&'a Icon>,
    parent: Option<ControlHandle>
}

impl<'a> ExternCanvasBuilder<'a> {

    pub fn flags(mut self, flags: ExternCanvasFlags) -> ExternCanvasBuilder<'a> {
        self.flags = Some(flags);
        self
    }

    pub fn title(mut self, text: &'a str) -> ExternCanvasBuilder<'a> {
        self.title = text;
        self
    }

    pub fn size(mut self, size: (i32, i32)) -> ExternCanvasBuilder<'a> {
        self.size = size;
        self
    }

    pub fn position(mut self, pos: (i32, i32)) -> ExternCanvasBuilder<'a> {
        self.position = pos;
        self
    }

    pub fn icon(mut self, ico: Option<&'a Icon>) -> ExternCanvasBuilder<'a> {
        self.icon = ico;
        self
    }

    pub fn parent<C: Into<ControlHandle>>(mut self, p: Option<C>) -> ExternCanvasBuilder<'a> {
        self.parent = p.map(|p2| p2.into());
        self
    }

    pub fn build(self, out: &mut ExternCanvas) -> Result<(), NwgError> {
        use winapi::um::winuser::{WS_CHILD};

        let mut flags = self.flags.map(|f| f.bits()).unwrap_or(out.flags());

        // Remove window flags if a parent is set
        if self.parent.is_some() {
            flags &= !(WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_THICKFRAME | WS_MAXIMIZEBOX);
            flags |= WS_CHILD;
        }

        out.handle = ControlBase::build_hwnd()
            .class_name(out.class_name())
            .forced_flags(out.forced_flags())
            .flags(flags)
            .size(self.size)
            .position(self.position)
            .text(self.title)
            .parent(self.parent)
            .build()?;

        if self.icon.is_some() {
            out.set_icon(self.icon);
        }

        Ok(())
    }

}