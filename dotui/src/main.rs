use dotcore::ipc::connection::IpcConnection;
use dotcore::events::{tennis};

#[tokio::main]
async fn main() { 

    println!("Dotui"); 
     
    let mut daemon_connection = IpcConnection::connect().await.expect("Failed to establish connection with daemon");
    
    let ping_message = tennis::Ping { msg: String::from("ping") };
    let pong_message = tennis::Pong { reply: String::from("pong") };


    match daemon_connection.send_frame(&ping_message).await {
        Ok(_) => println!("Ping message sent succesfully"),
        Err(e) => eprintln!("Failed to send a message: {e}")
    }

    match daemon_connection.send_frame(&pong_message).await {
        Ok(_) => println!("Pong message sent succesfully"),
        Err(e) => eprintln!("Failed to send a message: {e}")
    }
}
