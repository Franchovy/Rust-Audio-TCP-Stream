mod client;
mod server;
mod wav;
mod beep;
mod audio_stream;
mod audio_buffer;

use std::thread;

const BEEP_TEST:bool = false;
const STREAM_TEST:bool = false;
const CLIENT_SERVER_TEST_BEEP:bool = false;
const CLIENT_SERVER_TEST_MIC:bool = true;
const CLIENT2_TEST:bool = true;


#[allow(unreachable_code)]
fn main() {

    if BEEP_TEST {
        println!("Beep!");
        beep::beep();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if STREAM_TEST {
        println!("Testing stream.");

        std::thread::spawn(|| {
            audio_stream::audio_stream_test(5.0);
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
    }


    if CLIENT_SERVER_TEST_BEEP {
        println!("Running beep through Tcp protocol: ");
        std::thread::sleep(std::time::Duration::from_millis(100));


        println!("Running server.");

        std::thread::spawn(|| {
            server::run_server();
        });

        std::thread::sleep(std::time::Duration::from_millis(100));

        println!("Running client 1.");

        std::thread::spawn(|| {
            client::run_client(false);
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
    }


    if CLIENT_SERVER_TEST_MIC {
        println!("Now running audio stream through TCP.");
        std::thread::sleep(std::time::Duration::from_millis(20));
        println!("(WATCH OUT FOR FEEDBACK!)");
        std::thread::sleep(std::time::Duration::from_millis(100));

        println!("Running server.");

        std::thread::spawn(|| {
            server::run_server();
        });

        std::thread::sleep(std::time::Duration::from_millis(100));

        println!("Running client 1.");

        std::thread::spawn(|| {
            client::run_client(true);
        });

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    //infinite loop.
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if CLIENT2_TEST {
        println!("Running client 2.");

        std::thread::spawn(|| {
            client::run_client(false);
        });
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
