use std::{io, thread, time::Duration};

use flem::Status;

const PACKET_SIZE: usize = 512;

fn main() {
    let mut flem_serial = flem_serial_rs::FlemSerial::<PACKET_SIZE>::new();

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

                input_buffer.clear();

                // Select the serial port to use
                match io::stdin().read_line(&mut input_buffer) {
                    Ok(characters) => {
                        match input_buffer.trim().parse::<usize>() {
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
                            Err(parse_error) => {
                                // Bad parse, repeat the selection
                                println!("Error parsing selection: {}", parse_error.to_string());
                            }
                        }
                    },
                    Err(read_line_error) => {
                        println!(
                            "Error reading line, exiting program: {}", 
                            read_line_error.to_string()
                        );
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
            println!("Error connecting to serial port {} with error, exiting program", 
                port_name
            );
            return;
        }
    }

    let uart_rx_thread_handle = flem_serial.listen();

    let mut packet = flem::Packet::<PACKET_SIZE>::new();
    packet.set_request(flem::Request::ID as u8);
    packet.pack();
    flem_serial.send(&packet);
    

    let uart_rx_thread_processor = thread::spawn(move || {
        let mut timeout = 0;
        let rx_queue = flem_serial.received_packet_queue();
        loop {
            match rx_queue.recv() {
                Ok(packet) => {
                    timeout = 0;
                    let packet_data = &packet.get_data();
                    match packet.get_request() {
                        flem::Request::EVENT => {
                            let mut float_data = Vec::<f32>::new();
                            for slice in packet_data.chunks(4) {
                                float_data.push(f32::from_le_bytes([
                                    slice[0],
                                    slice[1],
                                    slice[2],
                                    slice[3],
                                ]));
                            }
                            println!("Real: {}, Imag: {}", float_data[0], float_data[1]);
                        },
                        flem::Request::ID => {
                            let id: flem::DataId = flem::DataId::from(packet_data).unwrap();
                            println!("Flem Device: {:?}, packet size: {}", id.get_version(), id.get_max_packet_size());
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

    uart_rx_thread_handle.join().unwrap();
    uart_rx_thread_processor.join().unwrap();

}