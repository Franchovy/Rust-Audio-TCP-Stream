use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};

extern crate portaudio;
use portaudio as pa;

use ringbuf::RingBuffer;
const RINGBUFFER_SIZE:usize = 5000;

// Input Audio Parameters
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES: u32 = 256;
const CHANNELS: i32 = 2;
const INTERLEAVED: bool = true;

// Sine Wave Parameters
use std::f64::consts::PI;

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
                                let audio_msg_length:f32 = string
                                    .parse().unwrap();

                                println!("Length: {}.", audio_msg_length);

                                if choice.eq(b"sin") {
                                    println!("Choose play sine");

                                    stream_sine(&mut stream, audio_msg_length);
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

fn stream_to_tcp(stream: &mut TcpStream, mut duration:f32) {

}

fn stream_mic(stream: &mut TcpStream, mut duration: f32) -> Result<(), Box<std::error::Error>> {
    // Launch PortAudio
    let pa = pa::PortAudio::new()?;

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    let settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, FRAMES);

    // Once the countdown reaches 0 we'll close the stream.
    let mut count_down = duration as f64;
    let mut previous_time = None;

    // Message passing channel
    let (sender, receiver) = ::std::sync::mpsc::channel();

    // Create Circular buffer to stream audio through to TCP
    let audio_buffer = ringbuf::RingBuffer::<f32>::new(RINGBUFFER_SIZE);
    let (mut buffer_producer, mut buffer_consumer) = audio_buffer.split();

    // Define the Audio Callback
    let callback = move |pa::InputStreamCallbackArgs {
                             buffer,
                             frames,
                             time,
                             ..
                         }| {
        let current_time = time.current;
        let prev_time = previous_time.unwrap_or(current_time);
        let dt = current_time - prev_time;
        count_down -= dt;
        previous_time = Some(current_time);

        assert!(frames == FRAMES as usize);
        sender.send(count_down).ok();

        // Push audio to the RingBuffer
        buffer_producer.push_slice(buffer);

        if count_down > 0.0 {
            println!("Receiving mic input...");
            pa::Continue
        } else {
            println!("Finished mic input.");
            pa::Complete
        }
    };

    // Construct the audio stream
    let mut audio_stream = pa.open_non_blocking_stream(settings, callback)?;

    // Set up the Tcp Stream buffer
    const BUFFER_LENGTH:usize = 1000;
    let mut data:[f32;BUFFER_LENGTH / 4] = [0.0; BUFFER_LENGTH / 4];

    // Start the audio input stream
    audio_stream.start()?;

    // Loop while the non-blocking stream is active.
    while let true = audio_stream.is_active()? {
        // Transfer data from the RingBuffer to the TCP Stream !
        buffer_consumer.pop_slice(&mut data);
        stream.write(f32_to_u8(&data))?;

        // Pass countdown message to the msg channel.
        while let Ok(count_down) = receiver.try_recv() {
            println!("count_down: {:?}", count_down);
        }
    }

    // Stop the stream.
    audio_stream.stop()?;

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

fn f32_to_u8(floats: &[f32]) -> &[u8] {
    unsafe {
        let bytes = floats.align_to::<u8>();
        assert_eq!(bytes.0.len() + bytes.2.len(), 0);
        assert_eq!(bytes.1.len(), floats.len() * 4);
        bytes.1
        //std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4)
    }
}