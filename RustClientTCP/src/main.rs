// Feito por Wesley Brandão

use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use sha2::{Sha256, Digest};
use hex;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").expect("Failed to connect to server");
    println!("Conectado ao servidor");

    loop {
        println!("Digite uma requisição (Sair, Arquivo <NOME>, Chat):");
        let mut request = String::new();
        std::io::stdin().read_line(&mut request).expect("Failed to read from stdin");

        if request.trim().eq_ignore_ascii_case("Sair") {
            stream.write_all(request.as_bytes()).expect("Failed to write to socket");
            break;
        } else if request.starts_with("Arquivo") {
            stream.write_all(request.as_bytes()).expect("Failed to write to socket");
            receive_file(&mut stream);
        } else if request.trim().eq_ignore_ascii_case("Chat") {
            stream.write_all(request.as_bytes()).expect("Failed to write to socket");
            start_chat(&mut stream);
        }
    }
}

fn receive_file(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    let mut file_name = String::new();
    let mut file_size = 0;
    let mut hash = String::new();

    let mut hasher = Sha256::new();
    let mut bytes_received = 0;
    let mut file = File::create(".buffer").expect("Failed to create file");

    // Recebe informações do arquivo
    while let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut cut = 0;
        for line in response.lines() {
            if line.starts_with("Nome:") {
                file_name = line.trim_start_matches("Nome: ").to_string();
                file = File::create(&file_name).expect("Failed to create file");
                cut += 7 + file_name.len();
            } else if line.starts_with("Tamanho:") {
                file_size = line.trim_start_matches("Tamanho: ").parse().unwrap();
                cut += 10 + file_size.to_string().len();
            } else if line.starts_with("Hash:") {
                hash = line.trim_start_matches("Hash: ").to_string();
                cut += 7 + hash.len();
            } else {
                let mut end = 0;

                if response.contains("Status: ok") {
                    end = 11;
                }

                let tosave = String::from_utf8_lossy(&buffer[cut..(bytes_read - end)]);
                println!("\nCut: {} Received {}", cut, bytes_read);
                
                file.write_all(&buffer[cut..(bytes_read - end)]).expect("Failed to write to file");
                hasher.update(&buffer[cut..(bytes_read - end)]);
                bytes_received += bytes_read - cut - end;
                break;
            }
        }
        if response.contains("Status: ok") {
            break;
        }

    }

    let hash_calculated = hex::encode(hasher.finalize());
    println!("{} and {}", bytes_received, file_size);
    if hash_calculated == hash {
        println!("Arquivo {} recebido e verificado com sucesso", file_name);
    } else {
        println!("Falha na verificação do arquivo {}", file_name);
    }
}

fn start_chat(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    loop {
        let mut msg = String::new();
        std::io::stdin().read_line(&mut msg).expect("Failed to read from stdin");
        stream.write_all(msg.as_bytes()).expect("Failed to write to socket");

        let bytes_read = stream.read(&mut buffer).expect("Failed to read from socket");
        if bytes_read == 0 {
            break;
        }
        let response = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("{}", response);
    }
}
