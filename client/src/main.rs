
extern crate serde;
use std::io::{self, ErrorKind, Read, Write, stdout};
use std::net::{TcpStream};
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    username: String,
    message: String,
}
const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 100;
fn print(input: String){
    print!("{}",input);
    let _flush = stdout().flush();

}
fn prompt(input: &str) -> String{
    print(input.to_owned());
    let stdin = io::stdin(); // We get `Stdin` here.
    let mut input = String::new();
    stdin.read_line(&mut input).expect("failed to read from stdin");
    input = input.trim().to_string();
    return input
}

fn main(){
    let username = prompt("Enter username: ");
    let username_c = username.clone();
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("Failed to initialize non-blocking");
    let (tx,rx) = std::sync::mpsc::channel::<String>();
    thread::spawn(move || loop{
        let mut buff = vec![0; MSG_SIZE];
        let msg_result = client.read_exact(&mut buff);
        match msg_result {
            Ok(_) => {
                
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                let message: Message = serde_json::from_str(&msg).expect("failed to deserialize message");
                if message.username != username_c{
                    print!("{}", (8u8 as char));
                    println!("{}: {}", message.username, message.message);
                    print("$".to_string());
                }
                
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }
        match rx.try_recv(){
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Write failed");
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,

        }
        thread::sleep(Duration::from_millis(1000));
    });
    
    loop{
        let mut buff = String::new();
        let message = prompt("$");
        if message == ":quit" {
            break;
        }
        let message = Message { username: username.clone(), message };
       
        if tx.send(serde_json::to_string(&message).expect("failed to serialize message")).is_err(){
            break;
        }

    }
}