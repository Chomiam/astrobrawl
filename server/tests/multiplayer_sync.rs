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

#[tokio::test]
async fn test_guest_player_progression_persistence() {
    let pool = init_db(":memory:").expect("DB init failed");
    let guest_id = "guest_persisted_test_123";

    // 1. Initial join as new guest
    let player_rec = astrobrawl_server::db::get_or_create_guest_player(&pool, guest_id, "Chomiam").unwrap();
    assert_eq!(player_rec.credits, 1000);
    assert_eq!(player_rec.score, 0);

    // 2. Play game, level up, gain credits & cargo
    let mut ship = PlayerShip::new(player_rec.id, "Chomiam".to_string(), ShipClass::Combat, Vec2::ZERO);
    ship.credits = 8500;
    ship.score = 12000;
    ship.add_xp(650);
    assert!(ship.level >= 2);
    ship.cargo.prometium = 42;

    // 3. Save progression (e.g. on disconnect or periodic auto-save)
    let save_data = ship.to_save_data();
    astrobrawl_server::db::save_player_progression(&pool, player_rec.id, &save_data).unwrap();

    // 4. Reconnect with the same guest_id after page refresh
    let refreshed_rec = astrobrawl_server::db::get_or_create_guest_player(&pool, guest_id, "Chomiam").unwrap();
    assert_eq!(refreshed_rec.id, player_rec.id, "Player ID must be identical across refreshes");
    assert!(refreshed_rec.save_data.is_some(), "Saved progression must be preserved in DB");

    // 5. Restore saved progression
    let mut restored_ship = PlayerShip::new(refreshed_rec.id, "Chomiam".to_string(), ShipClass::Combat, Vec2::ZERO);
    let parsed_save: PlayerSaveData = serde_json::from_str(&refreshed_rec.save_data.unwrap()).unwrap();
    restored_ship.apply_save_data(&parsed_save);

    assert_eq!(restored_ship.credits, 8500, "Credits must be restored");
    assert_eq!(restored_ship.score, ship.score, "Score must be restored");
    assert_eq!(restored_ship.level, ship.level, "Level must be preserved");
    assert_eq!(restored_ship.cargo.prometium, 42, "Cargo must be preserved");
}
