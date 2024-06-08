#![no_std]
mod watcher;

// scene_ptr: Watcher::new(vec![0x019151F8, 0x48, 0x10])
// retro_alarm_clicked: Watcher::new(vec![0x0195D848, 0x08, 0xB0, 0xA8, 0x28, 0x141])
// game_time: Watcher::new(vec![0x0195D848, 0x08, 0xB0, 0xC0, 0x28, 0x130])

extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

use alloc::{format, string::ToString, vec};
use asr::{future::next_tick, print_message, time::Duration, timer::{self, TimerState}, Process};
use watcher::{StringWatcher, Watcher};

const PROCESS_NAME: &str = "SuperliminalSteam";
const MODULE_NAME: &str = "UnityPlayer.dylib";

asr::async_main!(stable);
asr::panic_handler!();

async fn main() {
    // TODO: Set up some general state and settings.
    
    loop {
        let process = Process::wait_attach(PROCESS_NAME).await;
        process
            .until_closes(async {
                match process.get_module_address(MODULE_NAME) {
                    Ok(unity_module) => {
                        print_message(&format!("Unity Module: {unity_module}"));
                        let mut game_time = Watcher::<f64>::new(vec![0x0195D848, 0x08, 0xB0, 0xC0, 0x28, 0x130], 0.0);
                        let mut scene = StringWatcher::new(vec![0x019151F8, 0x48, 0x10]);
                        let mut retro_alarm_clicked = Watcher::<u8>::new(vec![0x0195D848, 0x08, 0xB0, 0xA8, 0x28, 0x141], 0u8);
                        loop {
                            game_time.update(&process, unity_module.value());
                            scene.update(&process, unity_module.value());
                            retro_alarm_clicked.update(&process, unity_module.value());
                            timer::set_variable("Scene", &scene.current);

                            match timer::state() {
                                TimerState::NotRunning => {
                                    // -- Start
                                    if game_time.current > 0.0 && game_time.current != game_time.old {
                                        timer::start();
                                    }
                                },
                                TimerState::Running => { 
                                    timer::set_game_time(Duration::seconds_f64(game_time.current));

                                    if scene.changed() {
                                        if scene.current.starts_with("Assets/_Levels/_LiveFolder/Misc/LoadingScenes/") &&
                                            scene.old.starts_with("Assets/_Levels/_LiveFolder/ACT")
                                        {
                                            timer::split();
                                        }

                                        else if scene.current.ends_with("StartScreen_Live.unity") {
                                            timer::reset();
                                        }
                                    }

                                    if scene.current.ends_with("TestChamber_Live.unity") && game_time.decreased(){
                                        timer::reset();
                                    }

                                    if scene.current.ends_with("EndingMontage_Live.unity") && 
                                        retro_alarm_clicked.changed_from_to(0, 1) 
                                    {
                                        timer::split();
                                    }
                                },
                                _ => { },
                            }
                            next_tick().await;
                        }
                    }
                    Err(_) => { }
                }
            })
            .await;
    }
}
