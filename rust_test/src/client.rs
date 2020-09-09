use std::net::{TcpStream};
use std::f64::consts::PI;
use std::io::{Read, Write};
use std::str::from_utf8;

pub(crate) fn run_client() {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333.");

            stream.write(b"stream sin 10s");

            let expected_msg = b"stream";

            let mut data = [0; 6];

            match stream.read(&mut data) {
                Ok(size) => {
                    if size == 6 && &data == expected_msg
                    {
                        // Begin audio stream
                        stream_audio(stream);
                    } else {
                        println!("Received invalid msg.");
                    }
                },
                Ok(_) => {
                    println!("Received sizeless msg.");
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

// fn on connect
fn stream_audio (stream : TcpStream) {

    println!("Streaming!");
    // start pa-stream
    // read from tcp-stream to pa-stream
}


// fn from byte slice to float

//todo move to server
fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4)
    }
}