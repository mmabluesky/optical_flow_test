use serial2::SerialPort;
use std::path::PathBuf;
use std::{thread, time::Duration};
use tauri::{AppHandle, Manager, Window};

// 数据结构来存储解析后的数据
struct OpticalFlowData {
    x: i16,
    y: i16,
}


// 定义一个全局变量存储port name
static mut PORT_NAME: String = String::new();


// 获取可用串口列表
fn get_serial_list() -> Vec<PathBuf> {
    match serial2::SerialPort::available_ports() {
        Err(e) => {
            eprintln!("Failed to enumerate serial ports: {}", e);
            std::process::exit(1);
        }
        Ok(ports) => {
            eprintln!("Found {} ports", ports.len());
            for port in &ports {
                // println!("{}", port.display())
            }
            ports
        }
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




// Tauri命令处理串行数据
#[tauri::command]
fn get_serial_data(window: Window, _app_handle: AppHandle) {
    println!("get_serial_data");

    // 在新线程中运行串口通信函数
    thread::spawn(move || {
        let port_name; // 串口名称
        let baud_rate = 115200; // 波特率

        let ports = get_serial_list();
        if ports.len() == 0 {
            println!("No serial port found");
            return;
        }
        port_name = ports[0].display().to_string();
        println!("Using port '{}'", port_name);

        //给全局变量PORT_NAME赋值
        unsafe {
            PORT_NAME = port_name.clone();
        }

        let port = match SerialPort::open(&port_name, baud_rate) {            
            Ok(port) => port,
            Err(e) => {
                eprintln!("Error opening serial port: {}", e);
                return; // Exit the thread if we can't open the port
            }
        };

        loop {
            let mut buffer = [0; 1]; // Buffer to read a single byt
            port.read_exact(&mut buffer).unwrap(); // Read one byte
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
                        eprintln!("Error reading from serial port: {}", e);
                        break; // or handle the error as appropriate
                    }
                }
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
        .invoke_handler(tauri::generate_handler![get_serial_data, get_port_name])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
