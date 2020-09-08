mod client;
mod server;
mod audio_test;

//use client;
//use server;
//use audio_test;



fn main() {
    println!("Hello, world!");

    std::thread::spawn(|| {
        server::run_server();
    } );

    client::run_client();
}
