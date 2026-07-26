// Copyright (C) 2026 JSalazarSoftware
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org>.

use evdev::{Device, InputEvent, KeyCode, EventType};
use evdev::uinput::VirtualDevice;
use std::error::Error;
use std::path::PathBuf;
use std::thread::{sleep, spawn};
use std::time::Duration;

// Helper to find ALL event paths belonging to the primary mouse
fn find_all_mouse_devices() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut enumerator = match udev::Enumerator::new() {
        Ok(e) => e,
        Err(_) => return paths,
    };
    
    if enumerator.match_subsystem("input").is_err() {
        return paths;
    }
    
    if let Some(devices) = enumerator.scan_devices().ok() {
        for device in devices {
            if let Some(properties) = device.property_value("ID_INPUT_MOUSE") {
                if properties == "1" {
                    if let Some(devnode) = device.devnode() {
                        if devnode.to_string_lossy().contains("event") {
                            paths.push(devnode.to_path_buf());
                        }
                    }
                }
            }
        }
    }
    paths
}

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        let mouse_paths = find_all_mouse_devices();
        if mouse_paths.is_empty() {
            sleep(Duration::from_secs(2));
            continue;
        }

        // 1. Open the first valid mouse device to copy its capabilities layout
        let template_device = match Device::open(&mouse_paths[0]) {
            Ok(dev) => dev,
            Err(_) => { sleep(Duration::from_secs(2)); continue; }
        };

        let supported_keys = template_device.supported_keys().unwrap_or_default();
        let supported_axes = template_device.supported_relative_axes().unwrap_or_default();

        // 2. Spin up our single virtual remapped mouse output
        let virtual_mouse: VirtualDevice = match VirtualDevice::builder() {
            Ok(builder) => match builder
                .name("Virtual Remapped Mouse")
                .with_keys(&supported_keys)?
                .with_relative_axes(&supported_axes)?
                .build() 
            {
                Ok(vm) => vm,
                Err(_) => { sleep(Duration::from_secs(2)); continue; }
            },
            Err(_) => { sleep(Duration::from_secs(2)); continue; }
        };

        // Make a thread-safe wrapper clone for the multi-device listeners
        let virtual_mouse_arc = std::sync::Arc::new(std::sync::Mutex::new(virtual_mouse));
        let mut worker_handles = vec![];

        println!("Found {} mouse sub-devices. Attempting global grab...", mouse_paths.len());

        // 3. Spawn a background listener thread for every single sub-stream found
        for path in mouse_paths {
            let mut physical_device = match Device::open(&path) {
                Ok(dev) => dev,
                Err(_) => continue,
            };

            if physical_device.grab().is_err() {
                println!("Warning: Could not grab sub-device {}", path.display());
                continue;
            }

            let vm_clone = std::sync::Arc::clone(&virtual_mouse_arc);
            
            let handle = spawn(move || {
                let key_type_raw = EventType::KEY.0;
                let sync_type_raw = EventType::SYNCHRONIZATION.0;

                loop {
                    let events = match physical_device.fetch_events() {
                        Ok(evs) => evs,
                        Err(_) => break, // Device disconnected or reset
                    };

                    for event in events {
                        if event.event_type() == EventType::KEY {
                            if event.code() == KeyCode::BTN_MIDDLE.code() {
                                if event.value() == 1 {
                                    if let Ok(mut vm) = vm_clone.lock() {
                                        // Click 1
                                        let _ = vm.emit(&[
                                            InputEvent::new(key_type_raw, KeyCode::BTN_LEFT.code(), 1),
                                            InputEvent::new(sync_type_raw, 0, 0)
                                        ]);
                                        sleep(Duration::from_millis(15));
                                        let _ = vm.emit(&[
                                            InputEvent::new(key_type_raw, KeyCode::BTN_LEFT.code(), 0),
                                            InputEvent::new(sync_type_raw, 0, 0)
                                        ]);

                                        sleep(Duration::from_millis(60));

                                        // Click 2
                                        let _ = vm.emit(&[
                                            InputEvent::new(key_type_raw, KeyCode::BTN_LEFT.code(), 1),
                                            InputEvent::new(sync_type_raw, 0, 0)
                                        ]);
                                        sleep(Duration::from_millis(15));
                                        let _ = vm.emit(&[
                                            InputEvent::new(key_type_raw, KeyCode::BTN_LEFT.code(), 0),
                                            InputEvent::new(sync_type_raw, 0, 0)
                                        ]);
                                    }
                                }
                            } else {
                                if let Ok(mut vm) = vm_clone.lock() { let _ = vm.emit(&[event.clone()]); }
                            }
                        } else {
                            if let Ok(mut vm) = vm_clone.lock() { let _ = vm.emit(&[event.clone()]); }
                        }
                    }
                    sleep(Duration::from_millis(1));
                }
            });
            worker_handles.push(handle);
        }

        println!("All mouse streams grabbed successfully!");

        // Wait here. If any thread exits, it means a sub-device disconnected/reset.
        // We break out to perform a complete hardware rescan.
        for handle in worker_handles {
            let _ = handle.join();
        }
        println!("A mouse stream closed. Re-initializing multi-grab scanner...");
        sleep(Duration::from_secs(1));
    }
}
