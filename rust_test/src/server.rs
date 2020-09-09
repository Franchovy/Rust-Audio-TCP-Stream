use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};


//====================================================
// All this should be moved to client as the audio player.
extern crate portaudio;
use portaudio as pa;
use std::f64::consts::PI;
use crate::beep::beep;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 1;
const SAMPLE_RATE: f64 = 44100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 100;


fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            //println!("Server read from stream. Size: {} ", size);

            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

pub(crate) fn run_server() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");

    for result in listener.incoming() {
        match result {
            Ok(mut stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {

                    // Connection succeeded
                    let mut data = [0 as u8; 14]; // read 14 byte header
                    while match stream.read(&mut data) {
                        Ok(size) => {
                            if data.starts_with(b"stream") && data.ends_with(b"s") {
                                println!("Correct");
                                let choice = &data[7..10];

                                let string = std::str::from_utf8(&data[11..13]).unwrap();
                                // todo handle panic
                                let mut audio_msg_length:i32 = string
                                    .parse().unwrap();

                                println!("Length: {}.", audio_msg_length);

                                if choice.eq(b"sin") {
                                    println!("Choose play sine");


                                } else if choice.eq(b"mic") {
                                    println!("Choose play mic");


                                } else {
                                    println!("Fail");
                                }

                                false
                            } else {
                                println!("Incorrect");
                                false
                            }

                        },
                        Err(_) => {
                            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
                            stream.shutdown(Shutdown::Both).unwrap();
                            false
                        }
                    } {}

                    handle_client(stream);
                    //stream_sine(stream);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                // Connection Failed
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(100));

    // close the socket listener
    drop(listener);
}

fn stream_sine(mut stream: TcpStream, mut duration: i32) -> Result<(), pa::Error> {

    // Create sin table
    let mut sine = [0.0; TABLE_SIZE];
    for i in 0..TABLE_SIZE {
        sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 4.0).sin() as f32; // 2x freq sounds better...
    }

    const BUFFER_LENGTH:usize = 1000;

    // Write to stream
    let mut data = [0 as u8; BUFFER_LENGTH];

    let mut cont:bool = true;
    while cont {
        let size_left = fill_buffer_with_table_loop(&data, &sine, duration);
        duration = size_left;

        match stream.write(&*data) {
            Ok(_) => {
                println!("Write ok.");
                if duration <= 0 {
                    cont = false;
                }
            },
            Err(e) => {
                println!("Error: {}", e);
                cont = false;
            }
        }
    }

    /*while match stream.read(&mut data) {
        Ok(size) => {
            println!("Server read from stream. Size: {} ", size);

            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}*/

    Ok(())
}

/**
*   Returns size leftover.
**/
fn fill_buffer_with_table_loop(buffer: &mut[u8], table: &[f32], mut size_in_secs: i32) -> i32 {
    let table_u8 = f32_to_u8(table);
    let mut index = 0;

    const SAMPLE_RATE:i32 = 44100; //Assuming 44.1K sample rate
    let size_leftover = size_in_secs * SAMPLE_RATE - buffer.len() as i32;
    let mut table_len_in_secs = table.len() as i32 / SAMPLE_RATE;

    while size_in_secs > table_len_in_secs as i32
        && index + table.len() < buffer.len()
    {
        // Copy table
        buffer[index..].copy_from_slice(table_u8);

        size_in_secs -= table_len_in_secs as i32;
        index += table.len();
    }
    if size_in_secs as i32 > 0 {
        // Copy what's left of table
        buffer[index..].copy_from_slice(&table_u8[..buffer[index..].len()]);
    }

    size_leftover
}

fn f32_to_u8(v: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}