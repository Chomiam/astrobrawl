use astrobrawl_server::game::GameWorld;
use astrobrawl_server::db::init_db;
use astrobrawl_shared::*;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_multiplayer_world_sync() {
    let pool = init_db(":memory:").expect("DB init failed");
    let mut world = GameWorld::new(pool);

    // Player 1 joins
    let (tx1, mut rx1) = mpsc::unbounded_channel();
    let ship1 = PlayerShip::new(1, "Chomiam".to_string(), ShipClass::Combat, Vec2::new(0.0, 0.0));
    world.add_player(ship1, tx1);

    // Player 2 joins
    let (tx2, mut rx2) = mpsc::unbounded_channel();
    let ship2 = PlayerShip::new(2, "Alice".to_string(), ShipClass::Minier, Vec2::new(100.0, 50.0));
    world.add_player(ship2, tx2);

    // Run tick simulation
    world.tick_simulation();

    // Drain messages from Player 1 until WorldSnapshot is found
    let mut p1_snapshot = None;
    while let Ok(bytes) = rx1.try_recv() {
        if let Ok(msg) = deserialize_packet::<ServerMessage>(&bytes) {
            if let ServerMessage::WorldSnapshot(s) = msg {
                p1_snapshot = Some(s);
            }
        }
    }

    let snapshot = p1_snapshot.expect("Player 1 must receive a WorldSnapshot");
    assert_eq!(snapshot.players.len(), 2, "Snapshot should contain both players");
    assert!(snapshot.players.iter().any(|p| p.username == "Chomiam"), "Snapshot should have Chomiam");
    assert!(snapshot.players.iter().any(|p| p.username == "Alice"), "Snapshot should have Alice");
    assert_eq!(snapshot.aliens.len(), 6, "Map 1-1 should have exactly 6 Streuners");
    for alien in &snapshot.aliens {
        assert_eq!(alien.alien_type, AlienType::Streuner, "All aliens should be Streuners");
    }

    // Drain messages from Player 2 until WorldSnapshot is found
    let mut p2_snapshot = None;
    while let Ok(bytes) = rx2.try_recv() {
        if let Ok(msg) = deserialize_packet::<ServerMessage>(&bytes) {
            if let ServerMessage::WorldSnapshot(s) = msg {
                p2_snapshot = Some(s);
            }
        }
    }

    let p2_s = p2_snapshot.expect("Player 2 must receive a WorldSnapshot");
    assert_eq!(p2_s.players.len(), 2, "Snapshot should contain both players");
}
