
const BUFFER_SIZE:usize = 5000;

/**
The audio buffer is a circular buffer of f32 and of fixed size.
Use read() and write() to interact with the buffer.

//todo error handling:
//err: write on full
//err: read on empty

read_ptr and write_ptr are indexes for reading and writing and should not
be accessed directly.
**/
struct audio_buffer {
    buffer:[f32; BUFFER_SIZE],
    read_ptr:usize,
    write_ptr:usize
}

impl audio_buffer {
    fn read(&self, size:usize, read_buffer:&mut[f32]) {
        assert_eq!(read_buffer.len(), size);

        if self.read_ptr + size > BUFFER_SIZE {
            // Read looping back
            let first_half_write_ref = &mut read_buffer[..size - self.read_ptr];
            let first_half_read_ref = & self.buffer[self.read_ptr..];

            assert_eq!(first_half_write_ref.len(), first_half_read_ref.len());
            first_half_write_ref.copy_from_slice(first_half_read_ref);

            let second_half_write_ref = &mut read_buffer[size - self.read_ptr..];
            let second_half_read_ref = & self.buffer[..size - self.read_ptr];

            assert_eq!(second_half_read_ref.len(), second_half_write_ref.len());
            second_half_write_ref.copy_from_slice(second_half_read_ref);
        } else {
            // Read normal
            read_buffer.copy_from_slice(&self.buffer[self.read_ptr..self.read_ptr + size]);
        }
    }
    fn write(&mut self, size:usize, write_data:&[f32]) {
        assert_eq!(write_data.len(), size);

        if self.write_ptr + size > BUFFER_SIZE {
            // Read looping back
            let first_half_write_ref = &mut self.buffer[self.write_ptr..];
            let first_half_read_ref = & write_data[..size - self.write_ptr];

            assert_eq!(first_half_write_ref.len(), first_half_read_ref.len());
            first_half_write_ref.copy_from_slice(first_half_read_ref);

            let second_half_write_ref = &mut self.buffer[..size - self.write_ptr];
            let second_half_read_ref = & write_data[size - self.write_ptr..];

            assert_eq!(second_half_read_ref.len(), second_half_write_ref.len());
            second_half_write_ref.copy_from_slice(second_half_read_ref);
        } else {
            // Read normal
            self.buffer[self.write_ptr..self.write_ptr + size].copy_from_slice(write_data);
        }
    }
    /// Returns amount of elements ready to read
    fn size_filled(&self) -> usize {
        if self.write_ptr > self.read_ptr {
            // Size without loop
            self.write_ptr - self.read_ptr
        } else {
            // Size with loop
            BUFFER_SIZE - self.read_ptr + self.write_ptr
        }
    }
}