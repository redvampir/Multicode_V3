use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Message {
    content: String,
}

#[tokio::main]
async fn main() {
    let msg = Message {
        content: "Backend started".into(),
    };
    println!("{}", msg.content);

    tauri::async_runtime::spawn(async {
        println!("Tauri async runtime initialized");
    });

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
}
