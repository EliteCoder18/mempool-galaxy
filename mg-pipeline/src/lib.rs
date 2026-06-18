use futures_util::{StreamExt,SinkExt};
use tokio_tungstenite::{connect_async,tungstenite::protocol::Message};
use std::sync::{Arc,RwLock};
use mg_core::{MempoolState,Particle};
use serde_json::Value;

pub async fn run_websocket(state: Arc<RwLock<MempoolState>>){
    let url = "wss://www.blockchain.info/inv";

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect Websocket");
    let (mut write, mut read) = ws_stream.split();

    let sub_msg = serde_json::json!({"op":"unconfirmed_sub"}).to_string();
    write.send(Message::Text(sub_msg.into())).await.expect("Failed to send sub message");

    while let Some(msg) = read.next().await{
        if let Ok(Message::Text(text)) = msg{

            println!("Received message: { }", text);
            
            if let Ok(json) = serde_json::from_str::<Value>(&text){
                if json["op"] == "utx"{
            
            
                let size = json["x"]["size"].as_f64().unwrap_or(250.0) as f32;
                let fee_rate = (size % 50.0).max(1.0);
                    
                if let Ok(mut w_state) = state.write(){
                    let width  = w_state.screen_size.0;
                    let height = w_state.screen_size.1;

                    let start_x = if size % 2.0 == 0.0 { 0.0 } else {width};
                    let start_y = size%height;
                    w_state.particles.push(Particle{
                        pos:(start_x,start_y),
                        velocity:(0.0,0.0),
                        fee_rate,
                    });
                }

            }
        }
    }

 
}}