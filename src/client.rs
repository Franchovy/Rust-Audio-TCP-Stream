use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use byte_strings::concat_bytes;

extern crate portaudio;
use portaudio as pa;

use ringbuf;
use crate::server::f32_to_u8;

const RINGBUFFER_SIZE:usize = 5000;

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;
const CHANNELS: i32 = 1;
const INTERLEAVED: bool = true;
const OUTPUT_FRAMES_PER_BUFFER: u32 = 256;

pub(crate) fn run_client(mic_mode:bool, duration:i32) -> Result<(), Box::<dyn std::error::Error>> {
    TcpStream::connect("localhost:3333")?;

    let mut tcp_stream = result.unwrap();
    println!("Successfully connected to server in port 3333.");

    let mode = if mic_mode { "mic" } else { "sin" };
    let msg = format!("stream {} {:02}s", mode, duration);

    println!("Sending message: {}", msg);
    tcp_stream.write_all(msg.as_bytes());

    let mut dummy = [0;10];
    loop {
        // Wait to catch signal.
        if tcp_stream.peek(&mut dummy).is_ok() {
            break;
        }
    }
    // Begin audio stream
    stream_audio(tcp_stream, duration);
}

/// On connection with TCP Stream: this creates a PortAudio instance
/// and streams the TCP data through to it using a ringbuffer.
fn stream_audio (mut tcp_stream: TcpStream, duration:i32) -> Result<(), pa::Error> {

    // Allocate TCP buffer
    let mut tcp_buffer = [0 as u8; 300];

    // Allocate audio ringbuffer
    const AUDIO_BUFFER_LENGTH:usize = 50000;
    let (mut rb_producer, mut rb_consumer)
        = ringbuf::RingBuffer::<f32>::new(RINGBUFFER_SIZE).split();

    // Run TCP Listener
    let tcp_listener_handle = std::thread::spawn(move || {
        let mut time_out = false;
        while !time_out {
            if tcp_stream.read_exact(&mut tcp_buffer).is_ok() {
                // Fill audio buffer with floats
                rb_producer.push_slice(u8_to_f32(&mut tcp_buffer));
            } else {
                // Timeout check - 50ms waiting time
                for t in 50..0
                {
                    // Wait 1ms before check
                    std::thread::sleep(std::time::Duration::from_millis(1));

                    // Use peek to determine if any data has come through
                    let peek = tcp_stream.peek(&mut tcp_buffer); //inefficient
                    if peek.is_ok() && peek.unwrap() > 0 as usize { //todo err means peek @ nothing?
                        break;
                    } else if t == 0 {
                        // Timed out
                        println!("Timed out TCP stream.");
                        // todo return error
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

    let output_settings =
        pa.default_output_stream_settings::
        <f32>(CHANNELS, SAMPLE_RATE, OUTPUT_FRAMES_PER_BUFFER)?;

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

    // Construct output audio stream
    let mut output_stream
        = pa.open_non_blocking_stream(output_settings, output_stream_callback)?;
    output_stream.start()?;

    // plays for pre-programmed amount of time.
    println!("Play for {} seconds.", duration);
    pa.sleep(duration * 1_000 + 3);

    output_stream.stop()?;
    output_stream.close()?;

    Ok(())
}


// fn from byte slice to float
pub fn u8_to_f32(bytes: &[u8]) -> &[f32] {
    unsafe {
        let floats = bytes.align_to::<f32>();
        assert_eq!(floats.0.len() + floats.2.len(), 0);
        assert_eq!(floats.1.len() * 4, bytes.len());
        floats.1
    }
}