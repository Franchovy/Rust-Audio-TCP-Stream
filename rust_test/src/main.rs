mod client;
mod server;
mod audio_test;
mod beep;
mod stream;

use std::{thread, time};

fn main() {
    println!("Beep!");
    //beep::beep();

    //println!("Beep again!");
    //audio_test::audio_test();

    //let sleep_time = time::Duration::from_millis(10000);


    /*println!("Testing stream.");

    std::thread::spawn(|| {
        stream::main();
    });
    std::thread::sleep(sleep_time);*/


    println!("Running server.");
    std::thread::spawn(|| {
        server::run_server();
    } );

    println!("Running client 1.");

    std::thread::spawn(|| {
        client::run_client();
    } );

    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Running client 2.");

    std::thread::spawn(|| {
        client::run_client();
    } );
}
