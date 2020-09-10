/**
The audio buffer is a circular buffer of f32 and of fixed size.
Use read() and write() to interact with the buffer.

//todo error handling:
//err: write on full
//err: read on empty

// for now, because I don't understand how to allocate arrays dynamically in structs,
// the array is always 5000 long

read_ptr and write_ptr are indexes for reading and writing and should not
be accessed directly.
**/
pub struct AudioBuffer {
    buffer_size: usize,
    buffer: Box::<[f32; 5000]>,
    read_ptr: usize,
    write_ptr: usize
}

impl AudioBuffer {

    pub fn new(size: usize) -> AudioBuffer {
        AudioBuffer {
            buffer_size: size,
            buffer: Box::new([0.0;5000]),
            read_ptr: 0,
            write_ptr: 0
        }
    }

    pub fn read(&mut self, size: usize, read_buffer: &mut [f32]) {
        assert_eq!(read_buffer.len(), size);
        self.check_readable(size);

        if self.read_ptr + size > self.buffer_size {
            // Read looping back
            let first_half_write_ref = &mut read_buffer[..size - self.read_ptr];
            let first_half_read_ref = &self.buffer[self.read_ptr..self.buffer_size];

            assert_eq!(first_half_write_ref.len(), first_half_read_ref.len());
            first_half_write_ref.copy_from_slice(first_half_read_ref);

            let second_half_write_ref = &mut read_buffer[size - self.read_ptr..];
            let second_half_read_ref = &self.buffer[..size - self.read_ptr];

            assert_eq!(second_half_read_ref.len(), second_half_write_ref.len());
            second_half_write_ref.copy_from_slice(second_half_read_ref);

            // Reset read pointer
            self.read_ptr = second_half_read_ref.len();
        } else {
            // Read normal
            read_buffer.copy_from_slice(&self.buffer[self.read_ptr..self.read_ptr + size]);

            // Increment read pointer
            self.read_ptr += size;
        }
    }
    pub fn write(&mut self, size: usize, write_data: &[f32]) {
        assert_eq!(write_data.len(), size);
        self.check_writeable(size);

        if self.write_ptr + size > self.buffer_size {
            // Read looping back
            let first_half_write_ref = &mut self.buffer[self.write_ptr..self.buffer_size];
            let first_half_read_ref = &write_data[..size - self.write_ptr];

            assert_eq!(first_half_write_ref.len(), first_half_read_ref.len());
            first_half_write_ref.copy_from_slice(first_half_read_ref);

            let second_half_write_ref = &mut self.buffer[..size - self.write_ptr];
            let second_half_read_ref = &write_data[size - self.write_ptr..];

            assert_eq!(second_half_read_ref.len(), second_half_write_ref.len());
            second_half_write_ref.copy_from_slice(second_half_read_ref);

            // Reset write pointer
            self.write_ptr = second_half_write_ref.len();
        } else {
            // Read normal
            self.buffer[self.write_ptr..self.write_ptr + size].copy_from_slice(write_data);

            // Increment write pointer
            self.write_ptr += size;
        }
    }
    /// Returns amount of elements ready to read
    pub fn size_filled(&self) -> usize {
        if self.write_ptr > self.read_ptr {
            // Size without loop
            self.write_ptr - self.read_ptr
        } else {
            // Size with loop
            self.buffer_size - self.read_ptr + self.write_ptr
        }
    }

    fn check_readable (&self, size:usize) {
        if self.read_ptr <= self.write_ptr {
            // Check without loop
            assert!(self.read_ptr + size <= self.write_ptr);
        } else {
            // Check with loop
            assert!(self.read_ptr + size <= self.write_ptr + self.buffer_size);
        }
    }

    fn check_writeable (&self, size:usize) {
        if self.read_ptr < self.write_ptr {
            // Check without loop
            assert!(self.read_ptr >= self.write_ptr + size - self.buffer_size);
        } else {
            // Check with loop
            assert!(self.write_ptr + size <= self.read_ptr + self.buffer_size);
        }
    }
}
