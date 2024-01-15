use serial2::SerialPort;
use std::path::PathBuf;
use std::{thread, time::Duration};
use tauri::{AppHandle, Manager, Window};
use lazy_static::lazy_static;
use std::{
    // io::{BufRead, BufReader, Read, Write},
    sync::Mutex,
};

// 数据结构来存储解析后的数据
struct OpticalFlowData {
    x: i16,
    y: i16,
}


// 定义一个全局变量存储port name
static mut PORT_NAME: String = String::new();

lazy_static! {
    static ref SERIAL: Mutex<Option<SerialPort>> = Mutex::new(None);
}



// 获取可用串口列表
#[tauri::command]
fn get_serial_list() -> Result<Vec<PathBuf>, String> {
    match serial2::SerialPort::available_ports() {
        Err(e) => Err(format!("Failed to enumerate serial ports: {}", e)),
        Ok(ports) => Ok(ports),
    }
}



// Tauri命令获取当前串口名称
#[tauri::command]
fn get_port_name() -> String {
    println!("get_port_name");
    unsafe {
        PORT_NAME.clone()
    }
}


// Tauri命令设置当前串口名称，并返回操作结果
#[tauri::command]
fn set_port_name(port_name: String) -> bool {
    unsafe {
        PORT_NAME = port_name;
        !PORT_NAME.is_empty()
    }
}


//断开串口连接的命令
#[tauri::command]
fn close_serial() -> Result<(), String> {
    let mut guard = SERIAL.lock().unwrap();
    if let Some(port) = guard.take() {
        *guard = None;
        // port.close().map_err(|err| err.to_string())?; // 显式地关闭串口

        Ok(())
    } else {
        Err("Instrument not connected".to_string())
    }
}


// Tauri命令处理串行数据
#[tauri::command]
fn get_serial_data(window: Window, _app_handle: AppHandle) {

    // 在新线程中运行串口通信函数
    thread::spawn(move || {

        let baud_rate = 115200; // 波特率
        //等待串口名称设置完成
        while unsafe {
            PORT_NAME.len() == 0
        } {
            println!("wait for port name");
            thread::sleep(Duration::from_millis(100));
        }

        // 串口名称设置完成，获取串口名称
        let port_name = unsafe {
            PORT_NAME.clone()
        };

        println!("use port_name: {}", port_name);

        let  port = match SerialPort::open(&port_name, baud_rate) {            
            Ok(port) => port,
            Err(e) => {
                let error_message = format!("Error opening serial port: {}", e);
                eprintln!("{}", &error_message);
                window.emit("serial_error", &error_message).unwrap();
                return; // Exit the thread if we can't open the port
            }
        };

        let mut guard = SERIAL.lock().unwrap();
        *guard = Some(port.try_clone().unwrap());

        loop {
            let mut buffer = [0; 1]; // Buffer to read a single byt
            match port.read_exact(&mut buffer) {
                Ok(_) => {
                    // 成功读取数据
                    if buffer[0] == b'w' {
                        // 'w'等于0x77，用于标识数据包的开始
                        let mut code = [0; 13]; // Buffer for the main code
                        match port.read_exact(&mut code) {
                            Ok(_) => {
                                println!(
                                    "Raw bytes: {:?}",
                                    code.iter()
                                        .map(|b| format!("{:02x}", b))
                                        .collect::<Vec<String>>()
                                        .join(" ")
                                );
        
                                let package = &code[1..code.len() - 7];
                                let x = i16::from_le_bytes([package[0], package[1]]);
                                let y = i16::from_le_bytes([package[2], package[3]]);
        
                                // println!("{}: {:>6}, {:>6}, {:>6}", chrono::Utc::now().format("%H:%M:%S%.3f"), x, y, package[4]);
        
                                window
                                    .emit("serial_data", format!("{{\"x\": {}, \"y\": {}}}", x, y))
                                    .unwrap();
                               
                            }
                            Err(e) => {
                                let error_message = format!("Error reading from serial port: {}", e);
                                eprintln!("{}", &error_message);
                                window.emit("serial_error", &error_message).unwrap();
                                break; // or handle the error as appropriate
                            }
                        }
                    }

                },
                Err(e) => {
                    // 处理错误, e 是错误信息
                    let error_message = format!("Error reading from serial port: {}", e);
                    eprintln!("{}", &error_message);
                    window.emit("serial_error", &error_message).unwrap();
                    break; 

                },
            }
            thread::sleep(Duration::from_millis(20));
        }
    });
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_serial_data, 
                                                 get_port_name,
                                                 set_port_name,
                                                 get_serial_list,
                                                 close_serial
                                                ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
