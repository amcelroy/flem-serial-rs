use std::{thread::JoinHandle, thread, sync::{Arc, Mutex, mpsc::{self, Receiver}}, time::Duration};

use flem::Status;
use serialport::SerialPort;

enum HostSerialPortErrors {
    NoDeviceFoundByThatName,
    MultipleDevicesFoundByThatName,
    ErrorConnectingToDevice,
}

struct FlemSerial<const T: usize> {
    selected_port: String,
    baud: u32,
    tx_port: Option<Box<dyn SerialPort>>,
    rx_port: Option<Box<dyn SerialPort>>,
    rx_packet: Arc<Mutex<flem::Packet::<T>>>,
    tx_packet: Arc<Mutex<flem::Packet::<T>>>,
    received_packets: Option<Receiver<flem::Packet::<T>>>,
    rx_buffer: [u8; 64],
    continue_listening: Arc<Mutex<bool>>,
}

impl<const T: usize> FlemSerial<T> {
    pub fn new() -> Self {
        Self {
            selected_port: "".to_string(),
            baud: 115200,
            tx_port: None,
            rx_port: None,
            tx_packet: Arc::new(Mutex::new(flem::Packet::<T>::new())),
            rx_packet: Arc::new(Mutex::new(flem::Packet::<T>::new())),
            rx_buffer: [0; 64],
            received_packets: None,
            continue_listening: Arc::new(Mutex::new(false)),
        }
    }

    pub fn list_serial_ports(&self) -> Option<Vec<String>> {
        let mut vec_ports = Vec::new();

        let ports = serialport::available_ports();

        match ports {
            Ok(valid_ports) => {
                for port in valid_ports {
                    vec_ports.push(port.port_name);
                }
                return Some(vec_ports);
            }
            Err(error) => {
                print!("{}", error);
                return None;
            }
        }
    }

    pub fn received_packets(&mut self) -> &mut Receiver<flem::Packet<T>> {
        self.received_packets.as_mut().unwrap()
    }

    /// Attempts to connect to a serial port with a set baud.
    pub fn connect(&mut self, port_name: &String, baud: u32, ) -> Result<(), HostSerialPortErrors> {
        let ports =  serialport::available_ports().unwrap();

        let filtered_ports: Vec<_> = ports.iter()
            .filter(|port| port.port_name == *port_name)
            .collect();

        match filtered_ports.len() {
            0 => Err(HostSerialPortErrors::NoDeviceFoundByThatName),
            1 => {
                if let Ok(port) = serialport::new(port_name, baud).timeout(Duration::from_millis(10)).open() {

                    self.tx_port = Some(port.try_clone().expect("Couldn't clone serial port for tx_port"));
                    self.rx_port = Some(port.try_clone().expect("Couldn't clone serial port for rx_port"));
    
                    return Ok(());
                } else {
                    return Err(HostSerialPortErrors::ErrorConnectingToDevice);
                }
                
            },
            _ => Err(HostSerialPortErrors::MultipleDevicesFoundByThatName)
        }
    }

    /// Spawns a new thread and listens for data on the 
    pub fn listen(&mut self) -> JoinHandle<()> 
    {
        *self.continue_listening.lock().unwrap() = true;
        
        let mut local_rx_port = self.tx_port.as_mut().unwrap().try_clone().expect("Cloning ");
        let rx_packet_clone = self.rx_packet.clone();
        let continue_listening_clone = self.continue_listening.clone();
        let (tx, rx) = mpsc::channel::<flem::Packet::<T>>();

        self.received_packets = Some(rx);

        let rx_thread_handle = thread::spawn(move || {
            let mut rx_buffer = [0 as u8; 64];

            while *continue_listening_clone.lock().unwrap() { 
                match local_rx_port.read(&mut rx_buffer){
                    Ok(bytes_to_read) => {
                        // Check if there are any bytes, if there are no bytes, 
                        // put the thread to sleep
                        if bytes_to_read == 0 {
                            thread::sleep(Duration::from_millis(10));
                        }else{
                            for i in 0..bytes_to_read {
                                if let Ok(mut rx_packet) = rx_packet_clone.lock() {
                                    match rx_packet.add_byte(rx_buffer[i]) {
                                        Status::PacketReceived => {
                                            tx.send(rx_packet.clone()).unwrap();
                                            rx_packet.reset_lazy();
                                        },
                                        Status::PacketBuilding => {
                                            // Normal, building packet
                                        },
                                        Status::HeaderBytesNotFound => {
                                            rx_packet.reset_lazy();
                                        }
                                        _ => {
                                            tx.send(rx_packet.clone()).unwrap();
                                            rx_packet.reset_lazy();
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Err(error) => {
                        println!("Error with serial port: {}", error);
                        //break;
                    }
                }
            }

            // Set to false, just incase the Error function caused an early exit
            *continue_listening_clone.lock().unwrap() = false;
        });

        rx_thread_handle
    }

    pub fn unlisten(&mut self) {
        *self.continue_listening.lock().unwrap() = false;
    }

}

#[cfg(test)]
mod tests {
    use std::{sync::{Arc, Mutex}, time::Duration, thread};
    use crate::FlemSerial;

    #[test]
    fn test_list_serial_ports() {
        let mut flem_serial = FlemSerial::<512>::new();

        let ports = flem_serial.list_serial_ports().unwrap();
        print!("{:?}", ports);
        let result = flem_serial.connect(&ports[4], 115200);
        match result {
            Ok(()) => {
                let thread_handle = flem_serial.listen();

                // let listener_handle = thread::spawn(move || {
                //     // Handle the incoming packets
                //     flem_serial.get_received_packets().iter()
                // });
                
                for i in 0..100 {
                    thread::sleep(Duration::from_millis(10));
                }

                let mut valid_packets = 0;
                loop {
                    match flem_serial.received_packets().recv() {
                        Ok(packet) => {
                            let x = packet.get_request();
                            valid_packets += 1;
                        },
                        Err(error) => {
                            let error = error;
                            println!("{}", error);
                        }
                    }
                }



                flem_serial.unlisten();

                thread_handle.join().unwrap();
            },
            Err(error) => {

            }
        }
    }
}