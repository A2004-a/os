//! NovaOS Apex — Window Server & UI Input Event Model

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEvent {
    MouseMove { x: i32, y: i32, buttons: u8 },
    MouseDown { x: i32, y: i32, button: u8 },
    MouseUp { x: i32, y: i32, button: u8 },
    KeyDown { scancode: u8, key: u8 },
    KeyUp { scancode: u8, key: u8 },
    Char(char),
    Focus,
    Blur,
    Close,
    Resize { width: u32, height: u32 },
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserEvent {
    pub event_type: u32,
    pub arg1: i32,
    pub arg2: i32,
    pub arg3: u32,
    pub arg4: u32,
}

pub const EVENT_MOUSE_MOVE: u32 = 1;
pub const EVENT_MOUSE_DOWN: u32 = 2;
pub const EVENT_MOUSE_UP: u32 = 3;
pub const EVENT_KEY_DOWN: u32 = 4;
pub const EVENT_KEY_UP: u32 = 5;
pub const EVENT_CHAR: u32 = 6;
pub const EVENT_FOCUS: u32 = 7;
pub const EVENT_BLUR: u32 = 8;
pub const EVENT_CLOSE: u32 = 9;
pub const EVENT_RESIZE: u32 = 10;

impl From<WindowEvent> for UserEvent {
    fn from(ev: WindowEvent) -> Self {
        match ev {
            WindowEvent::MouseMove { x, y, buttons } => Self {
                event_type: EVENT_MOUSE_MOVE,
                arg1: x,
                arg2: y,
                arg3: buttons as u32,
                arg4: 0,
            },
            WindowEvent::MouseDown { x, y, button } => Self {
                event_type: EVENT_MOUSE_DOWN,
                arg1: x,
                arg2: y,
                arg3: button as u32,
                arg4: 0,
            },
            WindowEvent::MouseUp { x, y, button } => Self {
                event_type: EVENT_MOUSE_UP,
                arg1: x,
                arg2: y,
                arg3: button as u32,
                arg4: 0,
            },
            WindowEvent::KeyDown { scancode, key } => Self {
                event_type: EVENT_KEY_DOWN,
                arg1: scancode as i32,
                arg2: key as i32,
                arg3: 0,
                arg4: 0,
            },
            WindowEvent::KeyUp { scancode, key } => Self {
                event_type: EVENT_KEY_UP,
                arg1: scancode as i32,
                arg2: key as i32,
                arg3: 0,
                arg4: 0,
            },
            WindowEvent::Char(c) => Self {
                event_type: EVENT_CHAR,
                arg1: c as u32 as i32,
                arg2: 0,
                arg3: 0,
                arg4: 0,
            },
            WindowEvent::Focus => Self {
                event_type: EVENT_FOCUS,
                arg1: 0,
                arg2: 0,
                arg3: 0,
                arg4: 0,
            },
            WindowEvent::Blur => Self {
                event_type: EVENT_BLUR,
                arg1: 0,
                arg2: 0,
                arg3: 0,
                arg4: 0,
            },
            WindowEvent::Close => Self {
                event_type: EVENT_CLOSE,
                arg1: 0,
                arg2: 0,
                arg3: 0,
                arg4: 0,
            },
            WindowEvent::Resize { width, height } => Self {
                event_type: EVENT_RESIZE,
                arg1: width as i32,
                arg2: height as i32,
                arg3: 0,
                arg4: 0,
            },
        }
    }
}
