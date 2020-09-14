mod client;
mod server;
mod wav;
mod beep;
mod audio_stream;
mod audio_buffer;

use std::thread;
use std::env;

const BEEP_TEST:bool = false;
const STREAM_TEST:bool = false;
const CLIENT_SERVER_TEST:bool = true;


#[allow(unreachable_code)]
fn main() {

    //=========================================
    // Set parameters getting arguments: [mic/sin mode, num seconds]
    let args: Vec<String> = env::args().collect();

    let mic_mode;
    let duration;
    if (args.len() == 2) {
        let arg_mode = &args[1];
        let arg_num_seconds = &args[2];

        // Mic/Sin mode argument
        if arg_mode.contains("sin") {
            mic_mode = false;
        } else if arg_mode.contains("mic") {
            mic_mode = true;
        } else {
            // Mic by default.
            mic_mode = true;
        }
        // Duration argument
        if let s = arg_num_seconds.parse::<u32>() {
            duration = std::cmp::min::<u32>(s.unwrap(), 99); // Arg should not be 3-digits
        } else {
            duration = 10;
        }
    } else {
        mic_mode = true;
        duration = 10;
    }

    //=========================================

    // TEST: Output a sine wave using PortAudio
    if BEEP_TEST {
        println!("Beep!");
        beep::beep();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // TEST: Stream input to output using PortAudio
    if STREAM_TEST {
        println!("Testing stream.");

        std::thread::spawn(|| {
            audio_stream::audio_stream_test(5.0);
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    //=========================================
    // Stream input to output using PortAudio and through a TCP Stream.
    if CLIENT_SERVER_TEST {
        println!("Now running audio stream through TCP.");
        println!("(WATCH OUT FOR FEEDBACK!)");
        std::thread::sleep(std::time::Duration::from_millis(100));

        println!("Running server.");
        let server_handle = std::thread::spawn(|| {
            server::run_server();
        });

        std::thread::sleep(std::time::Duration::from_millis(100));

        println!("Running client.");
        let client_handle = std::thread::spawn(move || {
            client::run_client(mic_mode, duration.clone() as i32);
        });

        // ========================
        // Block on waiting for response.
        let server_result = server_handle.join();
        println!("Server thread finished.");
        let client_result = client_handle.join();
        println!("Client thread finished.");

        // Output if everything went fine
        if server_result.is_ok() && client_result.is_ok() {
            println!("Looks like a success.");
        } else if server_result.is_err() && client_result.is_err() {
            println!("Both threads failed!");
        } else if server_result.is_err() {
            println!("Server thread failed!");
        } else if client_result.is_err() {
            println!("Client thread failed!");
        }
    }
}

fn test_audio_buffer() {
    //=======================================
    // Test 1 -> normal write and read

    let mut audio_buffer = audio_buffer::AudioBuffer::new(100);

    let write_data:[f32;10] = [1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0];
    audio_buffer.write(write_data.len(), &write_data);

    assert_eq!(audio_buffer.size_filled(), 10);

    let mut read_data = [0.0; 10];
    audio_buffer.read(read_data.len(), &mut read_data);

    for i in 0..10 {
        assert_eq!(write_data[i], read_data[i]);
    }

    //=======================================
    // Test 2 -> circular write and read

    let write_data:[f32;100] = [1.0; 100];
    audio_buffer.write(write_data.len(), &write_data);

    assert_eq!(audio_buffer.size_filled(), 100);

    let mut read_data:[f32; 100] = [0.0; 100];
    audio_buffer.read(read_data.len(), &mut read_data);

    for i in 0 .. 99 {
        assert_eq!(write_data[i], read_data[i]);
    }

    println!("Custom audio buffer tests passed.");
}
