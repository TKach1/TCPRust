// Feito por Wesley Brandão

use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use sha2::{Sha256, Digest};
use hex;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer).expect("Failed to read from socket");
        if bytes_read == 0 {
            return;
        }

        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        if request.starts_with("Sair") {
            println!("Cliente desconectado.");
            break;
        } else if request.starts_with("Arquivo") {
            let file_name = request.trim_start_matches("Arquivo ").trim();
            match send_file(&mut stream, file_name) {
                Ok(_) => println!("Arquivo enviado: {}", file_name),
                Err(e) => println!("Erro ao enviar arquivo: {}", e),
            }
        } else if request.starts_with("Chat") {
            println!("Chat iniciado com o cliente.");
            loop {
                let bytes_read = stream.read(&mut buffer).expect("Failed to read from socket");
                if bytes_read == 0 {
                    break;
                }
                let chat_msg = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Cliente: {}", chat_msg);
                let response = format!("Servidor: {}", chat_msg);
                stream.write_all(response.as_bytes()).expect("Failed to write to socket");
            }
        }
    }
}

fn send_file(stream: &mut TcpStream, file_name: &str) -> std::io::Result<()> {
    let mut file = File::open(file_name)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    let mut file_size = 0;

    // Calcula o hash e tamanho do arquivo
    while let Ok(n) = file.read(&mut buffer) {
        if n == 0 {
            break;
        }
        file_size += n;
        hasher.update(&buffer[..n]);
    }
    let hash = hasher.finalize();
    let hash_str = hex::encode(hash);

    // Envia informações do arquivo
    stream.write_all(format!("Nome: {}\n", file_name).as_bytes())?;
    stream.write_all(format!("Tamanho: {}\n", file_size).as_bytes())?;
    stream.write_all(format!("Hash: {}\n", hash_str).as_bytes())?;

    // Envia o arquivo
    file = File::open(file_name)?;
    while let Ok(n) = file.read(&mut buffer) {
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n])?;
    }

    stream.write_all(b"\nStatus: ok")?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:7878")?;
    let listener = Arc::new(listener);

    println!("Servidor ouvindo na porta 7878");

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(move || {
            handle_client(stream);
        });
    }
    Ok(())
}
