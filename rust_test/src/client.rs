use std::net::{TcpStream};
use std::f64::consts::PI;
use std::io::{Read, Write};
use std::str::from_utf8;

extern crate portaudio;
use portaudio as pa;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 1;
const SAMPLE_RATE: f64 = 44100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 100;

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
fn stream_audio (mut stream: TcpStream) -> Result<(), pa::Error> {

    let tcp_buffer = &mut [0 as u8; 300];
    stream.read(tcp_buffer);

    println!("Streaming!");

    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::stream_flags::CLIP_OFF;

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        for i in 0..frames {

        }
        pa::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000);

    stream.stop()?;
    stream.close()?;

    // start pa-stream
    // read from tcp-stream to pa-stream

    Ok(())
}


// fn from byte slice to float

//todo move to server
fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe {
        std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4)
    }
}