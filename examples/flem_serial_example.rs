use std::{io, thread, time::Duration};

use flem::Status;

fn main() {
    let mut flem_serial = flem_serial_rs::FlemSerial::<512>::new();

    let mut input_buffer = String::new();

    let mut selection_invalid = true;

    let mut selected_port: Option<String> = None;

    while selection_invalid {
        match flem_serial.list_serial_ports() {
            Some(ports) => {
                let mut line = 0;
                for port in ports.iter() {
                    println!("{}. {}", line, port);
                    line += 1;
                }
                println!("{}. Quit", line);
                println!("Select port: ");

                // Select the serial port to use
                match io::stdin().read_line(&mut input_buffer) {
                    Ok(_) => {
                        match input_buffer.parse::<usize>() {
                            Ok(selection) => {
                                if selection < line {
                                    // Selection is valid
                                    selection_invalid = false;
                                    selected_port = Some(ports[selection].clone());
                                }else if selection == line {
                                    // quit the program
                                    return;
                                }else{
                                    // Repeat the selection
                                }
                            },
                            Err(_) => {
                                // Bad parse, repeat the selection
                            }
                        }
                    },
                    Err(_) => {
                        return;
                    }
                }
            },
            None => {
                println!("No serial ports detected, press any key to quit...");
                match io::stdin().read_line(&mut input_buffer) {
                    Ok(_) => {
                        return;
                    },
                    Err(_) => {
                        return;
                    }
                }
            }
        }
    }
    
    let port_name = &selected_port.unwrap();
    match flem_serial.connect(port_name, 115200) {
        Ok(_) => {

        },
        Err(_) => {
            println!("Error connecting to serial port {} with error", 
                port_name
            );
        }
    }

    let uart_rx_thread_handle = flem_serial.listen();

    let uart_tx_thread_handle = thread::spawn(move || {
        let mut timeout = 0;
        let rx_queue = flem_serial.received_packet_queue();
        loop {
            match rx_queue.recv() {
                Ok(packet) => {
                    timeout = 0;
                    match packet.get_request() {
                        flem::Request::EVENT => {

                        },
                        flem::Request::ID => {

                        }
                        _ => {
                            println!("Unknown command");
                        }
                    }
                },
                Err(_) => {
                    timeout += 1;
                    thread::sleep(Duration::from_millis(1));
                    if timeout > 100 {

                    }
                }
            }
        }
    });

    uart_rx_thread_handle.join();
    uart_tx_thread_handle.join();

}