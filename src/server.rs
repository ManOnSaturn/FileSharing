use crate::BUFFER_SIZE;
use std::borrow::Cow;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

fn receive_file(stream: &mut TcpStream) -> std::io::Result<()> {
    // Read filename length
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let filename_len = u32::from_be_bytes(len_buf) as usize;

    // Read filename
    let mut filename_buf = vec![0u8; filename_len];
    stream.read_exact(&mut filename_buf)?;
    let filename = String::from_utf8_lossy(&filename_buf);

    // Read file size
    let mut size_buf = [0u8; 8];
    stream.read_exact(&mut size_buf)?;
    let file_size = u64::from_be_bytes(size_buf);

    println!("Receiving file: {} ({} bytes)", filename, file_size);

    // Create file
    let (tx, handle) = create_file(filename);

    // Receive file data
    let mut remaining = file_size;
    let mut buffer = [0u8; BUFFER_SIZE];

    while remaining > 0 {
        let to_read = std::cmp::min(remaining as usize, BUFFER_SIZE);
        let n = stream.read(&mut buffer[..to_read])?;
        let result = tx.send(FileData {
            data: buffer,
            actual_buffer_size: n,
        });
        if result.is_err() {
            return Ok(()); // TODO handle error here
        }
        remaining -= n as u64;
    }

    drop(tx);

    match handle.join() {
        Ok(Ok(())) => println!("File received successfully!"),
        Ok(Err(e)) => println!("IO error: {}", e),
        Err(_) => println!("Thread panicked"),
    }

    // Send acknowledgment
    stream.write_all(b"OK")?;

    Ok(())
}

struct FileData {
    data: [u8; BUFFER_SIZE],
    actual_buffer_size: usize,
}

fn create_file(filename: Cow<str>) -> (Sender<FileData>, JoinHandle<std::io::Result<()>>) {
    let (tx, rx) = std::sync::mpsc::channel::<FileData>();
    let filename = filename.into_owned();

    // Spawn writer thread
    let handle = std::thread::spawn(move || -> std::io::Result<()> {
        let mut file = File::create(format!("received_{}", filename))?;

        for file_data in rx {
            file.write_all(&file_data.data[..file_data.actual_buffer_size])?;
        }

        file.flush()?;
        Ok(())
    });
    (tx, handle)
}

fn send_file(stream: &mut TcpStream) -> std::io::Result<()> {
    // Read filename length
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let filename_len = u32::from_be_bytes(len_buf) as usize;

    // Read filename
    let mut filename_buf = vec![0u8; filename_len];
    stream.read_exact(&mut filename_buf)?;
    let filename = String::from_utf8_lossy(&filename_buf);

    println!("Sending file: {}", filename);

    // Check if file exists
    let path = Path::new(filename.as_ref());
    if !path.exists() {
        stream.write_all(b"ERR")?;
        return Ok(());
    }

    // Open file
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    // Send OK status
    stream.write_all(b"OK")?;

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

    println!("File sent successfully!");

    Ok(())
}

pub fn start_server(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Server listening on {}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    println!("Client connected: {}", stream.peer_addr()?);

    // Read command (SEND or GET)
    let mut cmd_buf = [0u8; 4];
    stream.read_exact(&mut cmd_buf)?;
    let command = String::from_utf8_lossy(&cmd_buf);

    match command.as_ref() {
        "SEND" => receive_file(&mut stream)?,
        "GET " => send_file(&mut stream)?,
        _ => println!("Unknown command: {}", command),
    }

    Ok(())
}
