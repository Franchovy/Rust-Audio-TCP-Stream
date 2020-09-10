use std::net::{TcpStream};
use std::io::{Read, Write};

extern crate portaudio;
use portaudio as pa;

use ringbuf::RingBuffer;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 1;
const SAMPLE_RATE: f64 = 44100.0;
const FRAMES_PER_BUFFER: u32 = 64;

pub(crate) fn run_client() {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333.");

            stream.write(b"stream sin 10s");

            let mut data = [0; 6];

            match stream.read(&mut data) {
                Ok(_) => {
                    // Begin audio stream
                    stream_audio(stream); //todo error
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

    let audio_buffer = RingBuffer::<f32>::new(AUDIO_BUFFER_LENGTH);
    let (mut buffer_producer, mut buffer_consumer) = audio_buffer.split();

    // Fill audio buffer with floats
    let result = stream.read(&mut tcp_buffer); // Length is for size f32
    if result.is_ok() {
        let len = result.unwrap() / 4;
        //todo loop this
        buffer_producer.push_slice(from_byte_slice(&mut tcp_buffer));

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
        // Copy buffer_from_stream to audio_buffer
        buffer_consumer.pop_slice(&mut buffer[..frames]);

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