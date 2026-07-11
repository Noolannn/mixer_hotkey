use std::{ffi::c_void, mem::MaybeUninit, ptr::null};

use windows::{Win32::{Foundation::{FALSE, HANDLE}, Media::Audio::{IAudioSessionControl, IAudioSessionControl2, IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume, MMDeviceEnumerator, eMultimedia, eRender}, System::{Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx}, LibraryLoader::GetModuleHandleA, Threading::{OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW}}, UI::{Input::{GetRawInputData, GetRawInputDeviceInfoA, GetRawInputDeviceList, HRAWINPUT, KeyboardAndMouse::{HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN, RegisterHotKey, UnregisterHotKey}, RAWINPUTDEVICELIST, RID_DEVICE_INFO, RID_HEADER, RIDI_DEVICEINFO, RIDI_DEVICENAME}, WindowsAndMessaging::{CreateWindowExA, GetMessageA, HWND_MESSAGE, MSG, RegisterClassA, RegisterClassExA, WM_INPUT, WM_KEYFIRST, WNDCLASSEXA, WS_BORDER, WS_CAPTION, WS_EX_APPWINDOW, WS_OVERLAPPEDWINDOW}}}, core::{HRESULT, Interface, PCSTR, PWSTR}};

pub fn get_raw_input_device_list() -> Vec<RAWINPUTDEVICELIST> {
    let mut device_number = 0;
    
    unsafe {
        GetRawInputDeviceList(None, &mut device_number, std::mem::size_of::<RAWINPUTDEVICELIST>() as u32);
    }
    
    println!("Found {} devices", device_number);
    let mut raw_input_device_list = vec![MaybeUninit::<RAWINPUTDEVICELIST>::uninit(); device_number as usize];
    
    unsafe {
        let res = GetRawInputDeviceList(Some(raw_input_device_list[0].as_mut_ptr()), &mut device_number, std::mem::size_of::<RAWINPUTDEVICELIST>() as u32);
        if res == u32::MAX {
            return get_raw_input_device_list();
        }
    }

    return raw_input_device_list.iter().map(|d| unsafe { d.assume_init() }).collect();
}

pub fn get_raw_input_device_info_a(handle: HANDLE) -> RID_DEVICE_INFO {
    let mut cb_size = std::mem::size_of::<RID_DEVICE_INFO>() as u32;
    unsafe {
        GetRawInputDeviceInfoA(Some(handle), RIDI_DEVICEINFO, None, &mut cb_size);
    }
    let mut device_info = MaybeUninit::<RID_DEVICE_INFO>::uninit();
    unsafe {
        GetRawInputDeviceInfoA(Some(handle), RIDI_DEVICEINFO, Some(device_info.as_mut_ptr() as *mut c_void), &mut cb_size);
        return device_info.assume_init();
    }
}

pub fn get_message() -> MSG {
    let mut lpmsg = MaybeUninit::<MSG>::uninit();
    unsafe {
        let res = GetMessageA(lpmsg.as_mut_ptr(), None, 0, 0);
        let lpmsg = lpmsg.assume_init();
        if lpmsg.message == WM_INPUT {
            // GetRawInputData(std::mem::transmute(lpmsg.lParam), RID_HEADER, pdata, pcbsize, cbsizeheader);
        }
        if res == FALSE {
            panic!();
        }
        return lpmsg;
    }
}

pub fn create_window_ex_a() {
    let res = unsafe { GetModuleHandleA(windows::core::PCSTR::default()) };
    dbg!(&res);
    let mut wndclass = WNDCLASSEXA::default();
    let mut class_name = "mywindow".to_owned();
    wndclass.lpszClassName = windows::core::PCSTR(class_name.as_mut_ptr());
    unsafe {
        RegisterClassExA(&wndclass as *const WNDCLASSEXA);
    }
    let mut window_name = "MyWindow".to_owned();
    unsafe {
        CreateWindowExA(
            WS_EX_APPWINDOW,
            wndclass.lpszClassName,
            windows::core::PCSTR(window_name.as_mut_ptr()),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            100,
            100,
            Some(HWND_MESSAGE),
            None,
            Some(res.unwrap().into()),
            None
        );
    }
}

pub fn register_hotkey(id: i32, modifier: HOT_KEY_MODIFIERS, key_code: u32) {
    unsafe {
        if RegisterHotKey(None, id, modifier, key_code).is_err() {
            panic!("Failed to register hotkey");
        }
    }
}

pub fn unregister_hotkey(id: i32) {
    unsafe {
        if UnregisterHotKey(None, id).is_err() {
            panic!("Failed to unregister hotkey");
        }
    }
}

#[derive(Debug)]
pub struct AudioSession {
    pid: u32,
    handle: HANDLE,
    pub name: String,
    session: IAudioSessionControl,
}

impl AudioSession {
    pub fn new(pid: u32, handle: HANDLE, name: String, session: IAudioSessionControl) -> Self {
        Self {
            pid,
            handle,
            name,
            session,
        }
    }

    pub fn get_mute(&self) -> Result<bool, WinApiError> {
        let volume: ISimpleAudioVolume = self.session.cast().unwrap();
        match unsafe { volume.GetMute() } {
            Ok(b) => return Ok(b.as_bool()),
            Err(_err) => Err(WinApiError::Other)
        }
    }

    pub fn set_mute(&self, mute: bool) -> Result<(), WinApiError> {
        let volume: ISimpleAudioVolume = self.session.cast().unwrap();
        match unsafe { volume.SetMute(mute, std::ptr::null()) } {
            Ok(_) => return Ok(()),
            Err(_err) => return Err(WinApiError::Other)
        }
    }

    pub fn get_volume(&self) -> Result<f32, WinApiError> {
        let volume: ISimpleAudioVolume = self.session.cast().unwrap();
        match unsafe { volume.GetMasterVolume() } {
            Ok(v) => return Ok(v),
            Err(_err) => Err(WinApiError::Other)
        }
    }

    pub fn set_volume(&self, new_vol: f32) -> Result<(), WinApiError> {
        let mut new_vol = new_vol;
        if new_vol < 0.0 {
            new_vol = 0.0;
        }
        if new_vol > 1.0 {
            new_vol = 1.0;
        }
        let volume: ISimpleAudioVolume = self.session.cast().unwrap();
        match unsafe { volume.SetMasterVolume(new_vol, std::ptr::null()) } {
            Ok(_) => return Ok(()),
            Err(_err) => return Err(WinApiError::Other)
        }
    }

    pub fn get_sessions() -> Result<Vec<Self>, WinApiError> {
        if unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_err() {
            return Err(WinApiError::Other);
        }
        let enumerator: IMMDeviceEnumerator = unsafe { 
            match CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_ALL
            ) {
                Ok(enumerator) => {
                    enumerator
                },
                Err(_err) => {
                    return Err(WinApiError::Other);
                }
            }
        };
        let device: IMMDevice = unsafe {
            match enumerator.GetDefaultAudioEndpoint(
                eRender,
                eMultimedia
            ) {
                Ok(d) => {
                    d
                },
                Err(_err) => {
                    return Err(WinApiError::Other);
                }
            }
        };
        let session_manager: IAudioSessionManager2 = unsafe {
            match device.Activate(
                CLSCTX_ALL,
                None
            ) {
                Ok(s) => {
                    s
                },
                Err(_err) => {
                    return Err(WinApiError::Other);
                }
            }
        };
        let enumerator = unsafe {
            match session_manager.GetSessionEnumerator() {
                Ok(e) => {
                    e
                },
                Err(_err) => {
                    return Err(WinApiError::Other);
                }
            }
        };
        let count = unsafe {
            match enumerator.GetCount() {
                Ok(c) => {
                    c
                },
                Err(_err) => {
                    return Err(WinApiError::Other);
                }
            }
        };
        let mut sessions = vec![];
        println!("Found {} sessions", count);
        for i in 0..count {
            let session: IAudioSessionControl = unsafe {
                match enumerator.GetSession(i) {
                    Ok(s) => s,
                    Err(_err) => return Err(WinApiError::Other)
                }
            };
            let session2: IAudioSessionControl2 = match session.cast() {
                Ok(s) => s,
                Err(_err) => return Err(WinApiError::Other)
            };
            let pid = unsafe {
                match session2.GetProcessId() {
                    Ok(p) => p,
                    Err(_err) => return Err(WinApiError::Other)
                }
            };
            if pid == 0 {
                // We don't manage the core process
                continue;
            }
            let p_handle = unsafe {
                match OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION,
                    true,
                    pid
                ) {
                    Ok(p) => p,
                    Err(_err) => return Err(WinApiError::Other)
                }
            };
            let mut buffer = vec![0u16; 512];
            let mut size = buffer.len() as u32;
            unsafe {
                match QueryFullProcessImageNameW(
                    p_handle,
                    PROCESS_NAME_WIN32,
                    PWSTR(buffer.as_mut_ptr()),
                    &mut size
                ) {
                    Ok(_) => (),
                    Err(_err) => return Err(WinApiError::Other)
                }
            };
            let bytes = buffer[..(size as usize)].to_vec();
            let process_name = String::from_utf16(&bytes).unwrap();
            let audio_session = AudioSession::new(pid, p_handle, process_name, session);
            sessions.push(audio_session);
        }
        Ok(sessions)
    }
}

#[derive(Debug)]
pub enum WinApiError {
    ErrorCode(i32),
    Other,
}

pub enum HotKeyAction {
    ToggleMute,
    ChangeVolume(f32),
}

pub struct HotKey {
    pub id: i32,
    modifier: HOT_KEY_MODIFIERS,
    key: u32,
    audio_sessions: Vec<AudioSession>,
    action: HotKeyAction,
}

impl HotKey {
    pub fn new(id: i32, modifier: HOT_KEY_MODIFIERS, key: u32, audio_sessions: Vec<AudioSession>, action: HotKeyAction) -> Self {
        register_hotkey(id, modifier, key);
        Self { id, modifier, key, audio_sessions, action }
    }

    pub fn exec(&self) -> Result<(), WinApiError> {
        match self.action {
            HotKeyAction::ToggleMute => {
                for s in &self.audio_sessions {
                    let old_mute = s.get_mute()?;
                    s.set_mute(!old_mute)?;
                }
            },
            HotKeyAction::ChangeVolume(delta) => {
                for s in &self.audio_sessions {
                    let old_volume = s.get_volume()?;
                    s.set_volume(old_volume + delta)?;
                }
            }
        }
        return Ok(())
    }
}

impl Drop for HotKey {
    fn drop(&mut self) {
        unregister_hotkey(self.id);
    }
}

pub fn modifier_from_u32(n: u32) -> Option<HOT_KEY_MODIFIERS> {
    let modifiers = vec![MOD_SHIFT, MOD_CONTROL, MOD_ALT, MOD_WIN];
    for m in modifiers {
        if m.0 == n {
            return Some(m);
        }
    }
    None
}