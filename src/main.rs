mod config;
mod utils;

use std::{sync::mpsc::{self, TryRecvError, channel}, thread, time::Duration};

use windows::Win32::UI::WindowsAndMessaging::{PostQuitMessage, PostThreadMessageA, WM_DESTROY, WM_HOTKEY, WM_QUIT};

use crate::{config::Config, utils::{AudioSession, get_current_thread_id, get_message, post_quit_message, post_quit_message_to_thread}};

fn ask_options() -> i32 {
    println!("1) List all audio sessions");
    println!("2) Load config and start hotkeys");
    let mut num = 0;
    loop {
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err() {
            continue;
        }
        let answer = &answer[0..1];
        match answer.parse::<i32>() {
            Ok(n) => {
                num = n;
                break;
            },
            Err(_) => {
                println!("Unknown result");
                continue;
            }
        };
    }
    return num;
}

fn run_hotkeys() {

    let (sender, receiver) = channel::<u32>();

    let handle = thread::spawn(move || {
        let thread_id = get_current_thread_id();
        sender.send(thread_id);

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
                    println!("Triggered hotkey ID : {}", id);
                    // let lParam = msg.lParam.0 as i32;
                    // let bytes = lParam.to_be_bytes();
                    // let high_word = u16::from_be_bytes([bytes[0], bytes[1]]);
                    // let low_word = u16::from_be_bytes([bytes[2], bytes[3]]);
                    // println!("High word : {}", high_word);
                    // println!("Low word : {}", low_word);
                    for hotkey in &hotkey_list {
                        if hotkey.id == id as i32 {
                            hotkey.exec();
                        }
                    }
                },
                WM_QUIT => {
                    println!("Exit hotkey loop");
                    return;
                },
                _ => {

                }
            }
        }
    });

    let thread_id = match receiver.recv() {
        Ok(id) => id,
        Err(_) => {
            println!("ERROR RETRIEVING THREAD ID");
            return;
        }
    };
    
    loop {

        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err() {
            continue;
        }
        let answer = &answer[0..1];
        if answer.chars().next().unwrap() == 'q' {
            println!("Post quit message");
            post_quit_message_to_thread(thread_id);
            break;
        }

        
    }
}

fn main() {
    loop {
        match ask_options() {
            1 => {
                let sessions = match AudioSession::get_sessions() {
                    Ok(s) => s,
                    Err(_) => {
                        println!("Unable to find any audio session");
                        return;
                    }
                };
                for s in sessions {
                    println!("Session : {}", s.name);
                }
            },
            2 => {
                run_hotkeys();
            },
            _ => {
                println!("No result");
            }
        }
    }
}