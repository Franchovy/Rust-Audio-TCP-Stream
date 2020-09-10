use std::net::{TcpStream};
use std::f64::consts::PI;
use std::io::{Read, Write};
use std::str::from_utf8;

extern crate ringbuffer;
use ringbuffer as rb;

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
                    // Begin audio stream
                    stream_audio(stream);
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

    // Allocate buffers
    let mut tcp_buffer = [0 as u8; 300];


    const AUDIO_BUFFER_LENGTH:usize = 5000;
    let mut audio_buffer = [0 as f32; AUDIO_BUFFER_LENGTH];
    let mut index:i32 = 0;

    // Fill audio buffer with floats
    let result = stream.read(&mut tcp_buffer); // Length is for size f32 //todo loop this
    if (result.is_ok()) {
        let len = result.unwrap() / 4;
        audio_buffer[..len].copy_from_slice(from_byte_slice(&mut tcp_buffer));
    } else {
        result.err();
    }

    println!("Creating audio stream on client side..");

    // Create Portaudio object
    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::stream_flags::CLIP_OFF;

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        //todo implement circular buffer
        if index + frames as i32 > AUDIO_BUFFER_LENGTH as i32 {
            index -= AUDIO_BUFFER_LENGTH as i32;
        }

        //test if index is negative
        if index < 0 {
            index = 0;
            assert_eq!(index as usize, 0);
        }

        // Copy buffer_from_stream to buffer
        buffer[..frames].copy_from_slice(&audio_buffer[index as usize..index as usize + frames]);
        index += frames as i32;

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
fn from_byte_slice(bytes: &[u8]) -> &[f32] {
    unsafe {
        std::slice::from_raw_parts(bytes.as_ptr() as *const f32, bytes.len() / 4)
    }
}