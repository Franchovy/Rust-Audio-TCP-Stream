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

struct AudioStream {
    is_input:bool,
    is_output:bool,
    pa:pa::PortAudio,
    duration:f64,
    input_stream:Option<pa::Stream<pa::stream::NonBlocking, pa::stream::Input<f32>>>,
    output_stream:Option<pa::Stream<pa::stream::NonBlocking, pa::stream::Output<f32>>>,
    msg_receiver: std::sync::mpsc::Receiver::<f64>
}

impl AudioStream {

    /// Creates a new AudioStream object, ready to stream using portaudio and an internal ringbuffer.
    /// if both is_input and is_output are true, then the AudioStream streams in -> out by itself.
    /// (...which is useful for testing)
    pub fn new(is_input:bool, is_output:bool) -> Result<AudioStream, pa::error::Error> {

        let pa =  pa::PortAudio::new()?;
        let (mut rb_producer, mut rb_consumer) = ringbuf::RingBuffer::<f32>::new(RINGBUFFER_SIZE).split();

        let mut input_stream = None;
        let mut output_stream = None;

        let mut duration = 5.0;

        // Create message channel
        let (msg_sender, msg_receiver) = ::std::sync::mpsc::channel();

        if is_input {
            let def_input = pa.default_input_device()?;
            let input_info = pa.device_info(def_input)?;

            // Construct the input stream parameters.
            let latency = input_info.default_low_input_latency;
            let input_params = pa::StreamParameters::<f32>::new(def_input, CHANNELS, INTERLEAVED, latency);

            pa.is_input_format_supported(input_params, SAMPLE_RATE)?;
            let mut input_settings = pa::InputStreamSettings::new(input_params, SAMPLE_RATE, INPUT_FRAMES_PER_BUFFER);

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
            input_stream = Some(pa.open_non_blocking_stream(input_settings, input_stream_callback)?);

        }
        if is_output {
            let mut output_settings =
                pa.default_output_stream_settings::<f32>(CHANNELS, SAMPLE_RATE, OUTPUT_FRAMES_PER_BUFFER)?;
            // we won't output out of range samples so don't bother clipping them.
            output_settings.flags = pa::stream_flags::CLIP_OFF;

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

            // Construct output audio stream
            output_stream = Some(pa.open_non_blocking_stream(output_settings, output_stream_callback)?);

        }


        Ok(AudioStream {
            is_input,
            is_output,
            pa,
            duration,
            input_stream,
            output_stream,
            msg_receiver
        })
    }

    pub fn stream(&mut self) {
        if self.is_input {
            assert!(self.input_stream.is_some());
            self.input_stream.as_mut().unwrap().start();
        }
        if self.is_output {
            assert!(self.output_stream.is_some());
            self.output_stream.as_mut().unwrap().start();
        }

        // Loop while the non-blocking stream is active.
        while (self.is_input && self.input_stream.as_mut().unwrap().is_active().unwrap())
            || (self.is_output && self.output_stream.as_mut().unwrap().is_active().unwrap())
        {
            // Write countdown message from msg channel.
            while let Ok(count_down) = self.msg_receiver.try_recv() {
                println!("count_down: {:?}", count_down);
            }
        }

        if self.is_input {
            self.input_stream.as_mut().unwrap().stop();
        }
        if self.is_output {
            self.output_stream.as_mut().unwrap().stop();
        }
    }

    /// This doesn't actually work, the init. function sets the data before.
    pub fn set_duration(&mut self, duration:f64) {
        self.duration = duration;
    }
}

///         ! ! ! ! ! ! ! ! ! ! ! !  WATCH OUT!
/// more like audio_SCREAM_test... watch your ears at the start and end.
pub fn audio_stream_test(duration:f64) -> Result<(), pa::error::Error> {
    //===============================================
    // using test
    let mut stream_test = AudioStream::new(true, true)?;

    stream_test.set_duration(duration); //doesn't work, see above.
    stream_test.stream();

    println!("test");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("test2");

    //let input_stream = AudioStream::new(true, false);
    //let output_stream = AudioStream::new(false, true);

    Ok(())
}

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

    input_stream.stop();
    output_stream.stop();

    Ok(())
}


