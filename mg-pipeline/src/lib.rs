use futures_util::{SinkExt, StreamExt};
use mg_core::{ConnectionStatus, MempoolState, Particle};
use serde_json::Value;
use std::sync::{Arc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub async fn run_websocket(state: Arc<RwLock<MempoolState>>) {
    let url = "wss://ws.blockchain.info/inv";

    let connect = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        connect_async(url),
    );
    let ws_stream = match connect.await {
        Ok(Ok((stream, _))) => stream,
        Ok(Err(e)) => {
            eprintln!("ws connect failed: {e}");
            if let Ok(mut w) = state.write() {
                w.connection_status = ConnectionStatus::Failed;
            }
            return;
        }
        Err(_) => {
            eprintln!("ws connect timed out");
            if let Ok(mut w) = state.write() {
                w.connection_status = ConnectionStatus::Failed;
            }
            return;
        }
    };

    if let Ok(mut w) = state.write() {
        w.connection_status = ConnectionStatus::Connected;
    }

    let (mut write, mut read) = ws_stream.split();

    let sub = serde_json::json!({"op": "unconfirmed_sub"}).to_string();
    if write.send(Message::Text(sub.into())).await.is_err() {
        return;
    }

    while let Some(Ok(Message::Text(text))) = read.next().await {
        let json: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if json["op"] != "utx" {
            continue;
        }

        let x = &json["x"];
        let size = x["size"].as_f64().unwrap_or(250.0) as f32;

        // fee = sum(inputs) - sum(outputs)
        let inputs: f32 = x["inputs"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|i| i["prev_out"]["value"].as_f64())
                    .sum::<f64>() as f32
            })
            .unwrap_or(0.0);
        let outputs: f32 = x["out"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|o| o["value"].as_f64())
                    .sum::<f64>() as f32
            })
            .unwrap_or(0.0);

        let fee_rate = if inputs > outputs && size > 0.0 {
            ((inputs - outputs) / size).max(1.0)
        } else {
            1.0
        };

        if let Ok(mut w) = state.write() {
            let (width, height) = w.screen_size;
            let cx = width / 2.0;
            let cy = height / 2.0;
            // spawn on outer ring; angle derived from txid first byte
            let byte = x["hash"]
                .as_str()
                .and_then(|h| u8::from_str_radix(&h[..2], 16).ok())
                .unwrap_or(0) as f32;
            let angle = (byte / 255.0) * std::f32::consts::TAU;
            let radius = (cx.min(cy) * 0.90).max(10.0);
            w.particles.push(Particle {
                pos: (cx + angle.cos() * radius, cy + angle.sin() * radius * 0.55),
                velocity: (0.0, 0.0),
                fee_rate,
            });
            if w.particles.len() > 800 {
                w.particles.remove(0);
            }
        }
    }
}