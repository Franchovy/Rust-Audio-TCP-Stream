mod client;
mod server;
mod audio_test;
mod beep;
mod stream;
mod audio_buffer;

use std::thread;

fn main() {
    println!("Beep!");
    beep::beep();

    //println!("Beep again!");
    //audio_test::audio_test();

    //let sleep_time = time::Duration::from_millis(10000);


    /*println!("Testing stream.");

    std::thread::spawn(|| {
        stream::main();
    });
    std::thread::sleep(sleep_time);*/

    //test_audio_buffer(); //doesn't pass tests.

    println!("Running server.");
    std::thread::spawn(|| {
        server::run_server();
    } );

    println!("Running client 1.");

    std::thread::spawn(|| {
        client::run_client();
    });

    loop {
        thread::sleep(std::time::Duration::from_millis(10));
    }

    /*std::thread::spawn(|| {
        client::run_client();
    } );*/

/*    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Running client 2.");

    std::thread::spawn(|| {
        client::run_client();
    } );*/
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
