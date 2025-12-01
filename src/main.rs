mod client;
mod server;

const BUFFER_SIZE: usize = 8192;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage:");
        println!("  Server mode: {} server [address:port]", args[0]);
        println!("  Send file:   {} send [address:port] [filename]", args[0]);
        println!("  Get file:    {} get [address:port] [filename]", args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "server" => {
            let addr = args.get(2).map(|s| s.as_str()).unwrap_or("127.0.0.1:8080");
            server::start_server(addr)?;
        }
        "send" => {
            if args.len() < 4 {
                println!("Usage: {} send [address:port] [filename]", args[0]);
                return Ok(());
            }
            client::send_file_to_server(&args[2], &args[3])?;
        }
        "get" => {
            if args.len() < 4 {
                println!("Usage: {} get [address:port] [filename]", args[0]);
                return Ok(());
            }
            client::get_file_from_server(&args[2], &args[3])?;
        }
        _ => println!("Unknown command: {}", args[1]),
    }

    Ok(())
}
