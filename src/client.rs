use crate::BUFFER_SIZE;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;

pub fn send_file_to_server(addr: &str, filename: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    println!("Connected to server");

    // Send SEND command
    stream.write_all(b"SEND")?;

    // Send filename length and filename
    let filename_bytes = filename.as_bytes();
    stream.write_all(&(filename_bytes.len() as u32).to_be_bytes())?;
    stream.write_all(filename_bytes)?;

    // Open and send file
    let file = File::open(filename)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    // Send file size
    stream.write_all(&file_size.to_be_bytes())?;

    // Send file data
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n])?;
    }

    // Wait for acknowledgment
    let mut ack = [0u8; 2];
    stream.read_exact(&mut ack)?;

    println!("File sent successfully!");

    Ok(())
}

pub fn get_file_from_server(addr: &str, filename: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect(addr)?;
    println!("Connected to server");

    // Send GET command
    stream.write_all(b"GET ")?;

    // Send filename length and filename
    let filename_bytes = filename.as_bytes();
    stream.write_all(&(filename_bytes.len() as u32).to_be_bytes())?;
    stream.write_all(filename_bytes)?;

    // Read status
    let mut status = [0u8; 2];
    stream.read_exact(&mut status)?;

    if status.as_slice() == b"ERR" {
        println!("File not found on server");
        return Ok(());
    }

    // Read file size
    let mut size_buf = [0u8; 8];
    stream.read_exact(&mut size_buf)?;
    let file_size = u64::from_be_bytes(size_buf);

    println!("Receiving file: {} ({} bytes)", filename, file_size);

    // Create file
    let file = File::create(format!("downloaded_{}", filename))?;
    let mut writer = BufWriter::new(file);

    // Receive file data
    let mut remaining = file_size;
    let mut buffer = [0u8; BUFFER_SIZE];

    while remaining > 0 {
        let to_read = std::cmp::min(remaining as usize, BUFFER_SIZE);
        let n = stream.read(&mut buffer[..to_read])?;
        writer.write_all(&buffer[..n])?;
        remaining -= n as u64;
    }

    writer.flush()?;
    println!("File downloaded successfully!");

    Ok(())
}
