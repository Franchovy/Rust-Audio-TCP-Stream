use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::f64::consts::PI;

extern crate portaudio;
use portaudio as pa;

use ringbuf;
const RINGBUFFER_SIZE:usize = 5000;

const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;
const CHANNELS: i32 = 1;
const INTERLEAVED: bool = true;
const INPUT_FRAMES_PER_BUFFER: u32 = 256;

// Sine Wave Parameters
const TABLE_SIZE: usize = 100;


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
                        Ok(_) => {
                            if data.starts_with(b"stream") && data.ends_with(b"s") {
                                println!("Correct");
                                let choice = &data[7..10];

                                let string = std::str::from_utf8(&data[11..13]).unwrap();
                                // todo handle panic
                                let audio_msg_length:f64 = string
                                    .parse().unwrap();

                                println!("Length: {}.", audio_msg_length);

                                if choice.eq(b"sin") {
                                    println!("Choose play sine");

                                    stream_sine(&mut stream, audio_msg_length as f32);
                                } else if choice.eq(b"mic") {
                                    println!("Choose play mic");

                                    stream_mic(&mut stream, audio_msg_length);
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

                    //handle_client(stream);
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

fn stream_mic(tcp_stream: &mut TcpStream, mut duration: f64) -> Result<(), Box<dyn std::error::Error>> {
    // Launch PortAudio
    let pa = pa::PortAudio::new()?;

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(
        def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    let mut input_settings = pa::InputStreamSettings::new(
        input_params, SAMPLE_RATE, INPUT_FRAMES_PER_BUFFER);

    // Create audio -> tcp ringbuffer
    let (mut rb_producer, mut rb_consumer)
        = ringbuf::RingBuffer::<f32>::new(RINGBUFFER_SIZE).split();

    // Create message channel
    let (msg_sender, msg_receiver) = ::std::sync::mpsc::channel();

    // Define callback -> send input stream into ringbuffer
    let input_stream_callback = move |pa::InputStreamCallbackArgs {
                                          buffer,
                                          frames,
                                          time,
                                          ..
                                      }| {
        duration -= frames as f64 / SAMPLE_RATE;
        assert_eq!(buffer.len(), frames);
        msg_sender.send(duration).ok();

        while rb_producer.is_full() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Push audio to the RingBuffer
        rb_producer.push_slice(buffer);

        if duration > 0.0 {
            println!("Input: {} frames", rb_producer.len());
            pa::Continue
        } else {
            println!("Finished mic input.");
            pa::Complete
        }
    };

    // Construct input audio stream
    let mut input_stream
        = pa.open_non_blocking_stream(input_settings, input_stream_callback)?;

    // Set up the Tcp Stream buffer
    const BUFFER_LENGTH:usize = 1000;
    let mut data:[f32;BUFFER_LENGTH / 4] = [0.0; BUFFER_LENGTH / 4];

    // Start the audio input stream
    input_stream.start()?;

    // Loop while the non-blocking stream is active.
    while let true = input_stream.is_active()? {
        // Transfer data from the RingBuffer to the TCP Stream !
        rb_consumer.pop_slice(&mut data);
        tcp_stream.write(f32_to_u8(&data))?;

        // Pass countdown message to the msg channel.
        while let Ok(count_down) = msg_receiver.try_recv() {
            println!("count_down: {:?}", count_down);
        }
    }

    // Stop the stream.
    input_stream.stop()?;

    Ok(())
}

fn stream_sine(stream: &mut TcpStream, mut duration: f32) -> std::io::Result<()> {
    // Create sin table
    let mut sine = [0.0; TABLE_SIZE];
    for i in 0..TABLE_SIZE {
        sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 4.0).sin() as f32; // 2x freq sounds better...
    }

    const BUFFER_LENGTH:usize = 1000;

    // Write to stream
    let data = &mut [0 as u8; BUFFER_LENGTH];

    loop {
        let size_left = fill_buffer_with_table_loop(&mut data[..], &sine, duration);
        duration = size_left;
        println!("duration left: {}", duration);

        stream.write(&data[..])?;

        if duration < 0.0 {
            // todo close message
            break;
        }
    }

    Ok(())
}


/**
*   Returns size leftover.
**/
fn fill_buffer_with_table_loop(buffer: &mut[u8], table: &[f32], mut size_in_secs: f32) -> f32 {
    let table_u8 = f32_to_u8(table);
    let mut index = 0;

    const SAMPLE_RATE:i32 = 44100; //Assuming 44.1K sample rate
    let size_leftover_in_secs = size_in_secs - (buffer.len() as f32 / SAMPLE_RATE as f32 / 4.0);
    let table_len_in_secs = table.len() as f32 / SAMPLE_RATE as f32;

    while size_in_secs > table_len_in_secs
        && index + table_u8.len() < buffer.len()
    {
        // Copy table
        buffer[index.. index + table_u8.len()].copy_from_slice(table_u8);

        size_in_secs -= table_len_in_secs;
        index += table.len();
    }
    if size_in_secs as i32 > 0 {
        // Copy what's left of table
        let leftover_len = buffer[index..].len();
        buffer[index..].copy_from_slice(&table_u8[..leftover_len]);
    }

    size_leftover_in_secs
}

pub fn f32_to_u8(floats: &[f32]) -> &[u8] {
    unsafe {
        let bytes = floats.align_to::<u8>();
        assert_eq!(bytes.0.len() + bytes.2.len(), 0);
        assert_eq!(bytes.1.len(), floats.len() * 4);
        bytes.1
    }
}