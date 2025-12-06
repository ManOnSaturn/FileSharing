use std::borrow::Cow;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

pub struct FileData {
    pub data: [u8; BUFFER_SIZE],
    pub actual_buffer_size: usize,
}

pub const BUFFER_SIZE: usize = 8192;

pub fn create_file_writer(
    filename: Cow<str>,
) -> (Sender<FileData>, JoinHandle<std::io::Result<()>>) {
    let (tx, rx) = std::sync::mpsc::channel::<FileData>();
    let filename = filename.into_owned();

    // Spawn writer thread
    let handle = std::thread::spawn(move || -> std::io::Result<()> {
        let mut file = File::create(format!("downloaded_{}", filename))?;

        for file_data in rx {
            file.write_all(&file_data.data[..file_data.actual_buffer_size])?;
        }

        file.flush()?;
        Ok(())
    });
    (tx, handle)
}

pub fn join_received_file_handle(handle: JoinHandle<std::io::Result<()>>) {
    match handle.join() {
        Ok(Ok(())) => println!("File received successfully!"),
        Ok(Err(e)) => println!("IO error: {}", e),
        Err(_) => println!("Thread panicked"),
    }
}
