mod config;
mod utils;

use windows::Win32::UI::{Input::KeyboardAndMouse::{MOD_SHIFT, VK_VOLUME_MUTE}, WindowsAndMessaging::WM_HOTKEY};

use crate::{config::{Config, Binding}, utils::{AudioSession, get_message, register_hotkey, unregister_hotkey}};

fn main() {
    let mut path = std::env::current_dir().unwrap();
    path.push("config.toml");
    dbg!(&path);
    let config = Config::load_from(&path).unwrap();

    dbg!(&config);

    let hotkey_list = config.create_hotkeys();

    loop {
        let msg = get_message();
        
        match msg.message {
            WM_HOTKEY => {
                let id = msg.wParam.0;
                println!("ID : {}", id);
                let lParam = msg.lParam.0 as i32;
                let bytes = lParam.to_be_bytes();
                let high_word = u16::from_be_bytes([bytes[0], bytes[1]]);
                let low_word = u16::from_be_bytes([bytes[2], bytes[3]]);
                println!("High word : {}", high_word);
                println!("Low word : {}", low_word);
                for hotkey in &hotkey_list {
                    if hotkey.id == id as i32 {
                        hotkey.exec();
                    }
                }
            },
            _ => {

            }
        }
    }
}