use serialport::*;
use std::io::Write;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use std::{io, thread};
use tauri::Manager;
use std::fs::File;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use std::path::PathBuf;

const PACKET_SIZE: usize = 17;
const START_BYTE_1: u8 = 0xA5;
const START_BYTE_2: u8 = 0x5A;
const END_BYTE: u8 = 0x01;

#[derive(Clone, serde::Serialize)]
struct Payload {
    counter: u8,
    data: Vec<i16>
    // Raw UTF-8 bytes
}

// list the ports and return a vector of strings
pub fn list_ports() -> Vec<String> {
    // get the ports from serialport::available_ports()
    let ports = serialport::available_ports().expect("No ports found!");
    // make a vecotor of strings then create an iterator of ports then map port names an clone
    // and collect them into the vector
    let port_list: Vec<String> = ports.iter().map(|p| p.port_name.clone()).collect();
    // return tfhe ports list
    return port_list;
}

// try to init the serial and return the port
pub fn init_port(port_path: String, baud_rate: u32) -> Result<Box<dyn SerialPort>> {
    println!("Opening port: {}, baud: {}", port_path, baud_rate);
    let port = serialport::new(port_path, baud_rate)
        .timeout(Duration::from_millis(1000))
        .open()?;

    // return port
    return Ok(port);
}

// clone the port and move it into the thread
pub fn start_clone_thread(
    app: tauri::AppHandle,
    mut port_clone: Box<dyn SerialPort>,
    is_thread_open: Arc<AtomicBool>,
) {
    // state_gaurd.thread_handler = Some(ThreadHandler { sender: sender });
    let mut serial_buf: Vec<u8> = vec![0; 1024];
    let mut accumulated_buffer: Vec<u8> = Vec::new();

    // move clone into thread
    thread::spawn(move || {
        // open thread
        is_thread_open.store(true, Ordering::Relaxed);
        println!("Thread spawned");
        while is_thread_open.load(Ordering::Relaxed) {
            match port_clone.read(serial_buf.as_mut_slice()) {
                Ok(size) => {
                    // let raw_data: Vec<u8> = serial_buf[..size].to_vec();
                    // println!("Received raw data: {:?}", raw_data);

                    // // emmit update to fronten
                    // app.emit_all("updateSerial", Payload { message: raw_data }).unwrap();
                    accumulated_buffer.extend_from_slice(&serial_buf[..size]);

                    // Process packets if we have enough bytes
                    while accumulated_buffer.len() >= PACKET_SIZE {
                        // Check for start bytes and end byte for each packet
                        if accumulated_buffer[0] == START_BYTE_1 && accumulated_buffer[1] == START_BYTE_2 {
                            if accumulated_buffer[PACKET_SIZE - 1] == END_BYTE {
                                // Extract the packet
                                let packet = accumulated_buffer.drain(..PACKET_SIZE).collect::<Vec<u8>>();
    
                                // Extract counter byte and 6x 2-byte data values
                                let counter = packet[3];
                                let data: Vec<i16> = (0..6).map(|i| {
                                    let idx = 4 + (i * 2); // 4 is where the data starts
                                    let high = packet[idx] as i16;
                                    let low = packet[idx + 1] as i16;
                                    (high << 8) | low // Combine the two bytes into a 16-bit value
                                }).collect();
    
                                // Send data to Tauri frontend
                                // tauri::Builder::default()
                                //     .setup(move |app| {
                                    println!("Received raw data: {:?}", counter);
                                        app.emit_all("updateSerial", Payload { counter: counter, data: data }).expect("Failed to send event");
                                      
                                //         Ok(())
                                //     }).run(tauri::generate_context!())?;
                            } else {
                                // Invalid end byte, skip the packet
                                accumulated_buffer.drain(..1);
                            }
                        } else {
                            // Invalid start bytes, skip
                            accumulated_buffer.drain(..1);
                        }
                }}
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => {
                    // clone the app
                    let app_clone = app.clone();
                    use crate::AppData;
                    let state = app_clone.state::<AppData>();
                    // unclock gaurd
                    let mut state_gaurd = state.0.lock().unwrap();
                    // set the port as an none
                    // clone state gaurd data
                    let port_path = state_gaurd.port_items.port_path.clone();
                    let baud_rate = state_gaurd.port_items.baud_rate.clone();
                    // set the port as none
                    state_gaurd.port = None;

                    state_gaurd.is_recording = false;
                    is_thread_open.store(false, Ordering::Relaxed);
                    // disconnect frontend
                    // app.emit_all("isConnected", Payload {message: "disconnected".to_string()}).unwrap();
                }
            }
        }
        println!("Terminating no record thread and now enabling...");
        // reenable thread
        is_thread_open.store(true, Ordering::Relaxed);
    });
}

pub fn start_record_on_port(
    app: tauri::AppHandle,
    mut port_clone: Box<dyn SerialPort>,
    is_thread_open: Arc<AtomicBool>,
    mut file: Option<File>,
    path: PathBuf,
) {
    let mut serial_buf: Vec<u8> = vec![0; 32];
    let mut start_time = SystemTime::now();

    thread::spawn(move || {
        is_thread_open.store(true, Ordering::Relaxed);
        println!("Record thread spawned");

        while is_thread_open.load(Ordering::Relaxed) {
            match port_clone.read(serial_buf.as_mut_slice()) {
                Ok(size) => {
                    let raw_data: Vec<u8> = serial_buf[..size].to_vec();
                    println!("Received raw data: {:?}", raw_data);

                    if let Some(ref mut file) = file {
                        // file.write_all(data_str.as_bytes()).expect("Could not write to file");
                    }

                    // app.emit_all("updateSerial", Payload { message: raw_data })
                        // .unwrap();
                }
                // Handle errors
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => {
                    // clone the app
                    let app_clone = app.clone();
                    use crate::AppData;
                    let state = app_clone.state::<AppData>();
                    // unlock gaurd
                    let mut state_gaurd = state.0.lock().unwrap();
                    // set the port as none
                    state_gaurd.port = None;
                    // toggle recording
                    state_gaurd.is_recording = false;
                    is_thread_open.store(false, Ordering::Relaxed);
                    // TODO move to backend 
                    // app.emit_all("isConnected", Payload {message: "disconnected".to_string()}).unwrap();
                    // app.emit_all("isRecording", Payload {message: "not recording".to_string()}).unwrap();
                }
            }

            // make a new file every 10 min
            if start_time.elapsed().unwrap() >= Duration::from_secs(600) {
                // Close the current file
                if let Some(old_file) = file.take() {
                    drop(old_file);
                }

                let formatted_date_time = Local::now().format("%Y-%m-%d_%H.%M.%S").to_string();
                let file_name = format!("SerialWizard_{}.txt", formatted_date_time);
                let file_path = path.join(&file_name);

                match File::create(&file_path) {
                    Ok(new_file) => {
                        file = Some(new_file);
                        // reset the timer
                        start_time = SystemTime::now();
                    }
                    Err(e) => {
                    // clone the app
                    let app_clone = app.clone();
                    use crate::AppData;
                    let state = app_clone.state::<AppData>();
                    // unlock gaurd
                    let mut state_gaurd = state.0.lock().unwrap();
                    // set the port as none
                    state_gaurd.port = None;
                    // toggle recording
                    state_gaurd.is_recording = false;
                    is_thread_open.store(false, Ordering::Relaxed);
                    // TODO move to backend 
                    // app.emit_all("isConnected", Payload {message: "disconnected".to_string()}).unwrap();
                    // app.emit_all("isRecording", Payload {message: "not recording".to_string()}).unwrap();
                    }
                }
            }
        }

        println!("Terminating record thread and now enabling...");
        is_thread_open.store(true, Ordering::Relaxed);
    });
}
pub fn write_serial<'a>(port: &'a mut Box<dyn SerialPort>, input: &'a str) -> Result<usize> {
    // add newline
    let newinput = String::from(input);
    // newinput.push_str("\n");

    // try to write
    let output = newinput.as_bytes();
    // check if the port was initialized correctly

    port.write(output)?;

    return Ok(output.len());
}
