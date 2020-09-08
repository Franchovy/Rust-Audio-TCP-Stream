//use proc_macro::quote_span;

mod client;
mod server;
mod audio_test;
mod beep;

//use client;
//use server;
//use audio_test;



fn main() {
    println!("Beep!");
    beep::beep();

    println!("Beep again!");
    audio_test::audio_test();

    println!("Checking server.");
    std::thread::spawn(|| {
        server::run_server();
    } );

    client::run_client();
}
