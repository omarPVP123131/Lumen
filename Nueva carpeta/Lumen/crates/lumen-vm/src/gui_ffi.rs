#![cfg(feature = "full")]
#![allow(clippy::missing_safety_doc, clippy::type_complexity)]
use std::collections::HashMap;
use std::sync::Mutex;

#[allow(clippy::upper_case_acronyms)]
type HWND = isize;

#[derive(Clone, Debug)]
pub enum GuiEvent {
    Create,
    Close,
    Destroy,
    Paint,
    Size { w: i32, h: i32 },
    Command { id: u16 },
    Char(u16),
    KeyDown(u16),
}

pub struct GuiWindow {
    pub hwnd: HWND,
    events: std::sync::mpsc::Receiver<GuiEvent>,
    pub should_close: bool,
}

struct GuiState {
    windows: HashMap<HWND, std::sync::mpsc::Sender<GuiEvent>>,
}

static GUI_STATE: Mutex<Option<GuiState>> = Mutex::new(None);

pub unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let state = GUI_STATE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(ref gui_state) = *state {
        if let Some(tx) = gui_state.windows.get(&hwnd) {
            let event = match msg {
                0x0001 => Some(GuiEvent::Create),
                0x000F => Some(GuiEvent::Paint),
                0x0010 => Some(GuiEvent::Close),
                0x0002 => Some(GuiEvent::Destroy),
                0x0100 => Some(GuiEvent::KeyDown(wparam as u16)),
                0x0102 => Some(GuiEvent::Char(wparam as u16)),
                0x0111 => Some(GuiEvent::Command {
                    id: (wparam & 0xFFFF) as u16,
                }),
                _ => None,
            };
            if let Some(evt) = event {
                let _ = tx.send(evt);
            }
        }
    }
    drop(state);
    unsafe {
        let Ok(user32) = libloading::Library::new("user32.dll") else {
            return 0;
        };
        let Ok(def) = user32.get::<unsafe extern "system" fn(HWND, u32, usize, isize) -> isize>(
            b"DefWindowProcA\0",
        ) else {
            return 0;
        };
        def(hwnd, msg, wparam, lparam)
    }
}

impl GuiWindow {
    pub fn create(title: &str, width: i32, height: i32) -> Result<Self, String> {
        unsafe {
            let user32 = libloading::Library::new("user32.dll").map_err(|e| e.to_string())?;

            // GetModuleHandleA(NULL)
            let kernel32 = libloading::Library::new("kernel32.dll").map_err(|e| e.to_string())?;
            let get_mod: libloading::Symbol<unsafe extern "system" fn(*const i8) -> isize> =
                kernel32
                    .get(b"GetModuleHandleA\0")
                    .map_err(|e| e.to_string())?;
            let hinst = get_mod(std::ptr::null());

            // RegisterClassExA
            let class_name = b"LumenGuiWindow\0"; // narrow string
            let mut wc = vec![0u8; 80];
            let p = wc.as_mut_ptr();
            *(p as *mut u32) = 80; // cbSize
            *(p.add(4) as *mut u32) = 0x0008 | 0x0020; // style: CS_HREDRAW | CS_VREDRAW
            *(p.add(8) as *mut i64) = wnd_proc as *const () as usize as i64; // lpfnWndProc
            *(p.add(16) as *mut i32) = 0; // cbClsExtra
            *(p.add(20) as *mut i32) = 0; // cbWndExtra
            *(p.add(24) as *mut i64) = hinst as i64; // hInstance
            *(p.add(32) as *mut i64) = 0; // hIcon
            *(p.add(40) as *mut i64) = 0; // hCursor
            *(p.add(48) as *mut i64) = 6; // hbrBackground = COLOR_WINDOW+1
            *(p.add(56) as *mut i64) = 0; // lpszMenuName
            *(p.add(64) as *mut i64) = class_name.as_ptr() as i64; // lpszClassName
            *(p.add(72) as *mut i64) = 0; // hIconSm

            let reg: libloading::Symbol<unsafe extern "system" fn(*const u16) -> u16> = user32
                .get(b"RegisterClassExA\0")
                .map_err(|e| e.to_string())?;
            let atom = reg(wc.as_ptr() as *const u16);
            if atom == 0 {
                return Err("RegisterClassExW failed".into());
            }

            // CreateWindowExA
            let create: libloading::Symbol<
                unsafe extern "system" fn(
                    u32,
                    *const i8,
                    *const i8,
                    u32,
                    i32,
                    i32,
                    i32,
                    i32,
                    HWND,
                    isize,
                    isize,
                    isize,
                ) -> HWND,
            > = user32
                .get(b"CreateWindowExA\0")
                .map_err(|e| e.to_string())?;

            let title_cs = std::ffi::CString::new(title).map_err(|e| e.to_string())?;
            let style = 0x00CF0000u32; // WS_OVERLAPPEDWINDOW

            let hwnd = create(
                0,
                class_name.as_ptr() as *const i8,
                title_cs.as_ptr().cast(),
                style,
                100,
                100,
                width,
                height,
                0,
                0,
                hinst,
                0,
            );
            if hwnd == 0 {
                return Err("CreateWindowExW failed".into());
            }

            let (tx, rx) = std::sync::mpsc::channel();
            {
                let mut state = GUI_STATE.lock().unwrap_or_else(|e| e.into_inner());
                if state.is_none() {
                    *state = Some(GuiState {
                        windows: HashMap::new(),
                    });
                }
                state.as_mut().unwrap().windows.insert(hwnd, tx);
            }

            Ok(GuiWindow {
                hwnd,
                events: rx,
                should_close: false,
            })
        }
    }

    pub fn show(&self) {
        unsafe {
            if let Ok(user32) = libloading::Library::new("user32.dll") {
                let show: libloading::Symbol<unsafe extern "system" fn(HWND, i32) -> bool> =
                    user32.get(b"ShowWindow\0").unwrap();
                show(self.hwnd, 5);
                let update: libloading::Symbol<unsafe extern "system" fn(HWND) -> bool> =
                    user32.get(b"UpdateWindow\0").unwrap();
                update(self.hwnd);
            }
        }
    }

    pub fn poll_event(&mut self) -> Option<GuiEvent> {
        unsafe {
            if let Ok(user32) = libloading::Library::new("user32.dll") {
                let peek: libloading::Symbol<
                    unsafe extern "system" fn(*mut u8, HWND, u32, u32, u32) -> bool,
                > = user32.get(b"PeekMessageW\0").unwrap();
                let translate: libloading::Symbol<unsafe extern "system" fn(*mut u8) -> bool> =
                    user32.get(b"TranslateMessage\0").unwrap();
                let dispatch: libloading::Symbol<unsafe extern "system" fn(*mut u8) -> isize> =
                    user32.get(b"DispatchMessageW\0").unwrap();

                let mut msg = vec![0u8; 48];
                while peek(msg.as_mut_ptr(), 0, 0, 0, 1) {
                    translate(msg.as_mut_ptr());
                    dispatch(msg.as_mut_ptr());
                }
            }
        }
        self.events.try_recv().ok()
    }

    pub fn hwnd(&self) -> isize {
        self.hwnd
    }
}

impl Drop for GuiWindow {
    fn drop(&mut self) {
        unsafe {
            if let Ok(user32) = libloading::Library::new("user32.dll") {
                if let Ok(destroy) =
                    user32.get::<unsafe extern "system" fn(HWND) -> bool>(b"DestroyWindow\0")
                {
                    destroy(self.hwnd);
                }
            }
        }
        let mut state = GUI_STATE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut s) = *state {
            s.windows.remove(&self.hwnd);
        }
    }
}
