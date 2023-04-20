use std::io;
fn main() {
    let mut flem_serial = flem_serial_rs::FlemSerial::<512>::new();

    let mut input_buffer = String::new();

    let mut selection_invalid = true;

    let mut selected_port: Option<usize> = None;

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
                                    selected_port = Some(selection);
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
    
    flem_serial.connect(ports[], baud);
}