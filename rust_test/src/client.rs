use std::net::{TcpStream};
use std::f64::consts::PI;
use std::io::{Read, Write};
use std::str::from_utf8;

pub(crate) fn run_client() {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333.");

            const sine_length:usize = 1024;
            let mut sine = [0.0; sine_length];
            for i in 0..sine_length {
                sine[i] = (i as f64 / sine_length as f64 * PI * 4.0).sin() as f32;
            }
            let mut msg =

            stream.write(to_byte_slice(&sine)).unwrap();

            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    println!("Reply is ok!");
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}

fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4)
    }
}