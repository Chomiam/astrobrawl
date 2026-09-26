use crate::auth::verify_jwt;
use crate::db::{get_or_create_guest_player, get_player_by_id, DbPool};
use crate::game::SharedWorld;
use astrobrawl_shared::*;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub struct AppState {
    pub world: SharedWorld,
    pub db_pool: DbPool,
    pub jwt_secret: String,
}

pub type SharedState = Arc<AppState>;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, params, state))
}

async fn handle_socket(socket: WebSocket, params: HashMap<String, String>, state: SharedState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Authenticate player via query parameter token or fallback to guest
    let token = params.get("token").cloned().unwrap_or_default();
    let (player_id, username, player_rec) = if !token.is_empty() && token != "guest" && token != "guest_offline" {
        match verify_jwt(&token, &state.jwt_secret) {
            Ok(claims) => {
                let p_id = claims.sub.parse::<u64>().unwrap_or(0);
                match get_player_by_id(&state.db_pool, p_id) {
                    Ok(Some(rec)) => (rec.id, rec.username.clone(), rec),
                    _ => {
                        let guest_uuid = uuid_simple();
                        let guest_name = format!("Pilote-{}", &guest_uuid[..4]);
                        let rec = get_or_create_guest_player(&state.db_pool, &guest_uuid, &guest_name)
                            .unwrap();
                        (rec.id, rec.username.clone(), rec)
                    }
                }
            }
            Err(_) => {
                let guest_uuid = uuid_simple();
                let guest_name = format!("Pilote-{}", &guest_uuid[..4]);
                let rec = get_or_create_guest_player(&state.db_pool, &guest_uuid, &guest_name)
                    .unwrap();
                (rec.id, rec.username.clone(), rec)
            }
        }
    } else {
        let guest_uuid = uuid_simple();
        let guest_name = format!("Pilote-{}", &guest_uuid[..4]);
        let rec = get_or_create_guest_player(&state.db_pool, &guest_uuid, &guest_name)
            .unwrap();
        (rec.id, rec.username.clone(), rec)
    };

    info!("Joueur connecté au WebSocket: ID={} Username={}", player_id, username);

    let mut ship = PlayerShip::new(player_id, username.clone(), ShipClass::Combat, Vec2::new(0.0, 0.0));
    ship.credits = player_rec.credits;
    ship.score = player_rec.score;

    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

    {
        let mut world = state.world.write().await;
        world.add_player(ship.clone(), tx.clone());
    }

    let auth_ok = ServerMessage::AuthSuccess {
        player_id,
        username: username.clone(),
        ship: ship.clone(),
        current_map: MapId::Map1_1,
    };
    if let Ok(bytes) = serialize_packet(&auth_ok) {
        let _ = ws_sender.send(Message::Binary(bytes.into())).await;
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            if ws_sender.send(Message::Binary(bytes.into())).await.is_err() {
                break;
            }
        }
    });

    let world_clone = state.world.clone();
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Binary(bytes)) => {
                    if let Ok(client_msg) = deserialize_packet::<ClientMessage>(&bytes) {
                        match client_msg {
                            ClientMessage::Input {
                                thrust,
                                target_angle,
                                move_vec,
                            } => {
                                let mut world = world_clone.write().await;
                                world.handle_input(player_id, thrust, move_vec, target_angle);
                            }
                            ClientMessage::Shoot => {
                                let mut world = world_clone.write().await;
                                world.handle_shoot(player_id);
                            }
                            ClientMessage::StartMining { mineral_id } => {
                                let mut world = world_clone.write().await;
                                world.handle_mining(player_id, mineral_id, true);
                            }
                            ClientMessage::StopMining => {
                                let mut world = world_clone.write().await;
                                world.handle_mining(player_id, 0, false);
                            }
                            ClientMessage::SelectClass { class } => {
                                let mut world = world_clone.write().await;
                                world.handle_select_class(player_id, class);
                            }
                            ClientMessage::UpgradeTalent { talent_index } => {
                                let mut world = world_clone.write().await;
                                world.handle_upgrade_talent(player_id, talent_index);
                            }
                            ClientMessage::SellCargo => {
                                let mut world = world_clone.write().await;
                                world.handle_sell_cargo(player_id);
                            }
                            ClientMessage::Respawn => {
                                let mut world = world_clone.write().await;
                                world.handle_respawn(player_id);
                            }
                            ClientMessage::Ping { client_time } => {
                                let pong = ServerMessage::Pong {
                                    client_time,
                                    server_time: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as u64,
                                };
                                if let Ok(pong_bytes) = serialize_packet(&pong) {
                                    let _ = tx_clone.send(pong_bytes);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    warn!("Erreur WebSocket client {}: {:?}", player_id, e);
                    break;
                }
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }

    info!("Joueur déconnecté du WebSocket: ID={}", player_id);
    {
        let mut world = state.world.write().await;
        world.remove_player(player_id);
    }
}

fn uuid_simple() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen();
    format!("{:08x}", n)
}
