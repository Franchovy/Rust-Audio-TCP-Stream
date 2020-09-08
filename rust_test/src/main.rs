mod client;
mod server;
mod audio_test;

//use client;
//use server;
//use audio_test;



fn main() {
    println!("Hello, world!");

    audio_test::audio_test();
    
    std::thread::spawn(|| {
        server::run_server();
    } );

    client::run_client();
}
