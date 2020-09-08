use std::f32::consts::PI;
use std::i16;
use std::fs::File;

//extern crate hound;
//extern crate portaudio;
use hound;
use portaudio as pa;

use std::io::stdin;

// Define buffer size
const BUFFER_SIZE: usize = 1024;

pub(crate) fn audio_test() -> Result<(), pa::Error> {

    //=================================================================================
    // Write WAV

    eprintln!("Checking for WAV file...");
    let wav_file = File::open("sine.wav");

    if wav_file.is_ok() {
        eprintln!("No need to create WAV file, already exists.");
    } else {
        eprintln!("Creating WAV file 'sine.wav':");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create("sine.wav", spec).unwrap();

        for t in (0 .. 44100).map(|x| x as f32 / 44100.0) {
            let sample = (t * 440.0 * 2.0 * PI).sin();
            let amplitude = i16::MAX as f32;
            writer.write_sample((sample * amplitude) as i16).unwrap();
        }

        writer.finalize().unwrap();

        eprintln!("File created.");
    }

    //=================================================================================
    // Read WAV

    eprintln!("Playing WAV file:");

    // Set up the WAV reader.
    let wav = hound::WavReader::open("sine.wav").expect("WAV reader open failed");
    let spec = wav.spec();
    eprintln!(
        "sample rate: {}, channels: {}, sample bits: {}, format: {:?}",
        spec.sample_rate,
        spec.channels,
        spec.bits_per_sample,
        spec.sample_format
    );
    let mut samples = wav.into_samples::<i32>();

    // Set up the stream
    let pa = pa::PortAudio::new()?;
    let settings = pa.default_output_stream_settings(
        1, // 1 channel
        spec.sample_rate as f64,
        BUFFER_SIZE as u32,
    )?;
    let mut stream = pa.open_blocking_stream(settings)?;
    stream.start()?;

    let mut done = false;
    while !done {
        let status = stream.write(BUFFER_SIZE as u32, |buffer| {
            assert_eq!(buffer.len(), BUFFER_SIZE);
            for b in buffer.iter_mut() {
                let s = if done {
                    0
                } else {
                    match samples.next() {
                        Some(s) => {
                            s.expect("bad sample during WAV read")
                        }
                        None => {
                            done = true;
                            0
                        }
                    }
                };
                *b = s;
            }
        });

        // On underflow - skip to next buffer
        match status {
            Ok(_) => (),
            Err(pa::Error::OutputUnderflowed) => {
                eprintln!("underflow");
                for _ in 0..BUFFER_SIZE {
                    let _ = samples
                        .next()
                        .expect("bad sample during underflow");
                }
            }
            _ => {
                status?;
            }
        }
    }

    eprintln!("Finished playing WAV file.");

    stream.stop()?;
    stream.close()?;
    Ok(())

}
