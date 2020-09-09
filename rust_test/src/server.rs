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
const SAMPLE_RATE: f64 = 44_100.0;
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

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                    //run_audio(stream);

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

fn run_audio(mut stream: TcpStream) -> Result<(), pa::Error> {
    // Create sin table
    let mut sine = [0.0; TABLE_SIZE];
    for i in 0..TABLE_SIZE {
        sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 4.0).sin() as f32;
    }
    let mut left_phase = 0;
    let mut right_phase = 0;

    // Set up portaudio stream
    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::stream_flags::CLIP_OFF;

    // Define audio callback
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut idx = 0;
        for _ in 0..frames {
            buffer[idx] = sine[left_phase];
            buffer[idx + 1] = sine[right_phase];
            left_phase += 1;
            if left_phase >= TABLE_SIZE {
                left_phase -= TABLE_SIZE;
            }
            right_phase += 3;
            if right_phase >= TABLE_SIZE {
                right_phase -= TABLE_SIZE;
            }
            idx += 2;
        }
        pa::Continue
    };

    // Start stream
    let mut pa_stream = pa.open_non_blocking_stream(settings, callback)?;

    pa_stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    pa_stream.stop()?;
    pa_stream.close()?;

    // Read from stream
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            println!("Server read from stream. Size: {} ", size);

            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}

    Ok(())
}

fn append_buffer_to_audio(mut incoming_array: &[u8]) {
    // Append this array to an audio buffer.

}