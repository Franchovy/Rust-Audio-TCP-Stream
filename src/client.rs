use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use byte_strings::concat_bytes;

extern crate portaudio;
use portaudio as pa;

use ringbuf;
const RINGBUFFER_SIZE:usize = 5000;

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;
const CHANNELS: i32 = 1;
const INTERLEAVED: bool = true;
const INPUT_FRAMES_PER_BUFFER: u32 = 256;
const OUTPUT_FRAMES_PER_BUFFER: u32 = 256;

//===========================================
// PARAMETERS
const NUM_SECONDS:i32 = 10;

pub(crate) fn run_client() {
    let result = TcpStream::connect("localhost:3333");
    if result.is_ok() {
        let mut stream = result.unwrap();
        println!("Successfully connected to server in port 3333.");


        let msg = format!("stream {} {:02}s", "mic", NUM_SECONDS);

        println!("Sending message: {}", msg);
        stream.write(msg.as_bytes());

        let mut data = [0; 6];

        //todo make this the stream loop!.
        match stream.read(&mut data) {
            Ok(_) => {
                // Begin audio stream
                stream_audio(stream); //todo error handling
            },
            Err(e) => {
                println!("Failed to receive data: {}", e);
            }
        }
    } else {
        println!("Error connection to server!");
    }
    println!("Terminated.");
}

fn stream_from_tcp (mut stream: TcpStream) {

}

// fn on connect
fn stream_audio (mut stream: TcpStream) -> Result<(), pa::Error> {

    // Allocate TCP buffer
    let mut tcp_buffer = [0 as u8; 300];
    // Allocate audio ringbuffer
    const AUDIO_BUFFER_LENGTH:usize = 50000;
    let (mut rb_producer, mut rb_consumer) = ringbuf::RingBuffer::<f32>::new(RINGBUFFER_SIZE).split();

    // Run TCP Listener
    let stream_read_to_audio = std::thread::spawn(move || {
        let mut time_out = false;
        while !time_out {
            if stream.read(&mut tcp_buffer).unwrap() > 0 { //todo check for timeout!
                // Fill audio buffer with floats
                rb_producer.push_slice(from_byte_slice(&mut tcp_buffer));
            } else {
                // Timeout check
                for mut t in 50..0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                    let peek = stream.peek(&mut tcp_buffer);
                    if peek.is_ok() && peek.unwrap() > 0 as usize {
                        break;
                    }
                    if t == 0 {
                        // Timed out
                        println!("Timed out TCP stream.");
                        time_out = true;
                    }
                }
                println!("Finished receiving TCP stream.");
                break;
            }
        }
    });


    println!("Creating audio stream on client side..");

    // Create Portaudio object
    let pa = pa::PortAudio::new()?;

    let mut settings =
        pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_RATE, OUTPUT_FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    settings.flags = pa::stream_flags::CLIP_OFF;

    // Define Output callback -> send ringbuffer into output stream
    let output_stream_callback = move |pa::OutputStreamCallbackArgs {
                                           buffer,
                                           frames,
                                           ..
                                       }| {
        // Copy buffer_from_stream to audio_buffer
        assert_eq!(buffer.len(), frames);
        let len = rb_consumer.pop_slice(&mut buffer[..frames]);

        if len > 0 {
            println!("Output: {} frames", rb_consumer.len());
            pa::Continue
        } else {
            for time_out in 10..0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
                if !rb_consumer.is_empty() {
                    break;
                }
                if time_out == 0 {
                    // Timed out
                    println!("Done playing.");
                    return pa::Complete
                }
            }
            pa::Continue
        }
    };

    let mut stream = pa.open_non_blocking_stream(settings, output_stream_callback)?;

    stream.start()?;

    println!("Play for {} seconds.", NUM_SECONDS);
    pa.sleep(NUM_SECONDS * 1_000 + 1);

    stream.stop()?; //todo make this relative to the TCP stream.
    stream.close()?;

    // start pa-stream
    // read from tcp-stream to pa-stream

    Ok(())
}


// fn from byte slice to float
fn from_byte_slice(bytes: &[u8]) -> &[f32] {
    unsafe {
        let floats = bytes.align_to::<f32>();
        assert_eq!(floats.0.len() + floats.2.len(), 0);
        assert_eq!(floats.1.len() * 4, bytes.len());
        floats.1
        //std::slice::from_raw_parts(bytes.as_ptr() as *const f32, bytes.len() / 4)
    }
}