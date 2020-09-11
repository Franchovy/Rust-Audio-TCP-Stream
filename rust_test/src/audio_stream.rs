//! A demonstration of constructing and using a non-blocking stream.
//!
//! Audio from the default input device is passed directly to the default output device in a duplex
//! stream, so beware of feedback!

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

pub fn audio_stream(mut duration:f64) -> Result<(), Box::<std::error::Error>>{
    //===============================================
    // Create input audio stream

    // Launch PortAudio
    let pa = pa::PortAudio::new()?;

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
    let input_settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, INPUT_FRAMES_PER_BUFFER);

    // Message passing channel
    let (sender, receiver) = ::std::sync::mpsc::channel();

    //===============================================
    // Set RB to fill with input stream

    // Create Circular buffer to stream audio through to TCP
    let audio_buffer = ringbuf::RingBuffer::<f32>::new(RINGBUFFER_SIZE);
    let (mut producer, mut consumer) = audio_buffer.split();

    // Define the Audio Input Callback
    let input_stream_callback = move |pa::InputStreamCallbackArgs {
                             buffer,
                             frames,
                             time,
                             ..
                         }| {
        duration -= frames as f64 / SAMPLE_RATE;
        assert_eq!(buffer.len(), frames);
        sender.send(duration).ok();

        while producer.is_full() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        // Push audio to the RingBuffer
        producer.push_slice(buffer);

        if duration > 0.0 {
            println!("Input: {} frames", producer.len());
            pa::Continue
        } else {
            println!("Finished mic input.");
            pa::Complete
        }
    };

    //===============================================
    // Create output audio stream

    let mut output_settings =
        pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_RATE, OUTPUT_FRAMES_PER_BUFFER)?;
    // we won't output out of range samples so don't bother clipping them.
    output_settings.flags = pa::stream_flags::CLIP_OFF;

    //===============================================
    // Set output stream to pop from RB

    // Define Output stream callback
    let output_stream_callback = move |pa::OutputStreamCallbackArgs {
                             buffer,
                             frames,
                             ..
                         }| {
        // Copy buffer_from_stream to audio_buffer
        assert_eq!(buffer.len(), frames);
        let len = consumer.pop_slice(&mut buffer[..frames]);

        if len > 0 {
            println!("Output: {} frames", consumer.len());
            pa::Continue
        } else {
            for time_out in 500..0 {
                std::thread::sleep(std::time::Duration::from_millis(1));
                if !consumer.is_empty() {
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

    // Construct the audio stream
    let mut input_stream = pa.open_non_blocking_stream(input_settings, input_stream_callback)?;
    let mut output_stream = pa.open_non_blocking_stream(output_settings, output_stream_callback)?;

    input_stream.start();
    output_stream.start();

    // Loop while the non-blocking stream is active.
    while let true = input_stream.is_active()? {
        // Write countdown message from msg channel.
        while let Ok(count_down) = receiver.try_recv() {
            println!("count_down: {:?}", count_down);
        }
    }

    //input_stream.stop();
    //output_stream.stop();

    Ok(())
}

//todo run input only / output only with separate ringbuffers
// test this case

fn run() -> Result<(), pa::Error> {
    let pa = pa::PortAudio::new()?;

    eprintln!("PortAudio:");
    eprintln!("version: {}", pa.version());
    eprintln!("version text: {:?}", pa.version_text());
    eprintln!("host count: {}", pa.host_api_count()?);

    let default_host = pa.default_host_api()?;
    eprintln!("default host: {:#?}", pa.host_api_info(default_host));

    let def_input = pa.default_input_device()?;
    let input_info = pa.device_info(def_input)?;
    eprintln!("Default input device info: {:#?}", &input_info);

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

    let def_output = pa.default_output_device()?;
    let output_info = pa.device_info(def_output)?;
    eprintln!("Default output device info: {:#?}", &output_info);

    // Construct the output stream parameters.
    let latency = output_info.default_low_output_latency;
    let output_params = pa::StreamParameters::new(def_output, CHANNELS, INTERLEAVED, latency);

    // Check that the stream format is supported.
    pa.is_duplex_format_supported(input_params, output_params, SAMPLE_RATE)?;

    // Construct the settings with which we'll open our duplex stream.
    let settings = pa::DuplexStreamSettings::new(input_params, output_params, SAMPLE_RATE, FRAMES);

    // Once the countdown reaches 0 we'll close the stream.
    let mut count_down = 10.0;

    // Keep track of the last `current_time` so we can calculate the delta time.
    let mut maybe_last_time = None;

    // We'll use this channel to send the count_down to the main thread for fun.
    let (sender, receiver) = ::std::sync::mpsc::channel();

    // A callback to pass to the non-blocking stream.
    let callback = move |pa::DuplexStreamCallbackArgs {
                             in_buffer,
                             out_buffer,
                             frames,
                             time,
                             ..
                         }| {
        let current_time = time.current;
        let prev_time = maybe_last_time.unwrap_or(current_time);
        let dt = current_time - prev_time;
        count_down -= dt;
        maybe_last_time = Some(current_time);

        assert!(frames == FRAMES as usize);
        sender.send(count_down).ok();

        // Pass the input straight to the output - BEWARE OF FEEDBACK!
        for (output_sample, input_sample) in out_buffer.iter_mut().zip(in_buffer.iter()) {
            *output_sample = *input_sample;
        }

        if count_down > 0.0 {
            pa::Continue
        } else {
            pa::Complete
        }
    };

    // Construct a stream with input and output sample types of f32.
    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    // Loop while the non-blocking stream is active.
    while let true = stream.is_active()? {
        // Do some stuff!
        while let Ok(count_down) = receiver.try_recv() {
            println!("count_down: {:?}", count_down);
        }
    }

    stream.stop()?;

    Ok(())
}