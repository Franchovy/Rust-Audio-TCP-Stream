# **rust_test**

> by Maxime Franchot, for somebody special.

### Server transmitting audio to a client through TCP in rust.

From microphone (portaudio input) to speaker (portaudio output), processed on separate server and client.

## Files: 
#### Main files
- **main.rs** launches the server and client. Also a testbench launcher, customizable at the top of the file.
- **server.rs** on a client connection starts a PortAudio instance and streams audio through a ringbuffer, and then through a TCPstream.
- **client.rs** contacts a server, and upon success, will stream input data through a ringbuffer and out through speakers with a PortAudio instance.

#### "Library" & test files 
- **audio_stream.rs** is an attempt at an "object" in rust.... very confusing !!! It passes a test but I am unable to actually use it. Hey, it's practice.
- **beep.rs** plays a beep using a sine wave and PortAudio output. ~*Sounds a lot nicer than the server-client beep, actually.*~
- **audio_buffer.rs** implementation of a circular buffer before realising RingBuf does it very well already. It's ok, it's more practice.
- **wav.rs** test using Hound to write and read Wav files. Did not implement in the client-server interaction.
