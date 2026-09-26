use astrobrawl_shared::{
    deserialize_packet, serialize_packet, ClientMessage, Laser, Mineral, MineralType, PlayerId,
    PlayerShip, ServerMessage, Vec2, MINERAL_RADIUS, SHIP_RADIUS, WORLD_HEIGHT, WORLD_WIDTH,
};
use macroquad::prelude::*;
use std::collections::HashMap;

fn to_mq(v: Vec2) -> macroquad::math::Vec2 {
    vec2(v.x, v.y)
}

// --- Native & WASM Network Bridge ---
#[cfg(target_arch = "wasm32")]
mod net {
    extern "C" {
        pub fn mq_ws_connect(url_ptr: *const u8, url_len: usize);
        pub fn mq_ws_is_connected() -> i32;
        pub fn mq_ws_send(ptr: *const u8, len: usize);
        pub fn mq_ws_recv(dest_ptr: *mut u8, max_len: usize) -> i32;
    }

    pub fn connect(url: &str) {
        unsafe { mq_ws_connect(url.as_ptr(), url.len()) };
    }

    #[allow(dead_code)]
    pub fn is_connected() -> bool {
        unsafe { mq_ws_is_connected() == 1 }
    }

    pub fn send(bytes: &[u8]) {
        unsafe { mq_ws_send(bytes.as_ptr(), bytes.len()) };
    }

    pub fn try_recv() -> Option<Vec<u8>> {
        let mut buf = vec![0u8; 65536];
        let len = unsafe { mq_ws_recv(buf.as_mut_ptr(), buf.len()) };
        if len > 0 {
            buf.truncate(len as usize);
            Some(buf)
        } else {
            None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod net {
    pub fn connect(_url: &str) {}
    pub fn is_connected() -> bool {
        true
    }
    pub fn send(_bytes: &[u8]) {}
    pub fn try_recv() -> Option<Vec<u8>> {
        None
    }
}

// --- Particle & Visual Effects ---
struct Particle {
    pos: Vec2,
    vel: Vec2,
    color: Color,
    lifetime: f32,
    max_lifetime: f32,
    size: f32,
}

struct FloatText {
    text: String,
    pos: Vec2,
    color: Color,
    lifetime: f32,
}

// --- Starfield Background ---
struct Star {
    pos: Vec2,
    size: f32,
    color: Color,
    layer: f32,
}

// --- Client Game State ---
struct GameClient {
    local_player_id: Option<PlayerId>,
    local_username: String,
    connected: bool,
    connecting: bool,
    status_text: String,

    players: HashMap<PlayerId, PlayerShip>,
    lasers: Vec<Laser>,
    minerals: Vec<Mineral>,
    server_tick: u64,

    camera_pos: Vec2,
    last_ping_send: f64,
    ping_ms: u64,

    particles: Vec<Particle>,
    float_texts: Vec<FloatText>,
    stars: Vec<Star>,
}

impl GameClient {
    fn new() -> Self {
        let mut stars = Vec::with_capacity(300);
        for i in 0..300 {
            let seed = (i as f32) * 12.9898;
            let x = ((seed.sin() * 43758.5453).fract() - 0.5) * WORLD_WIDTH * 1.4;
            let y = (((seed + 1.0).sin() * 43758.5453).fract() - 0.5) * WORLD_HEIGHT * 1.4;
            let layer = ((seed + 2.0).sin() * 43758.5453).fract() * 0.8 + 0.2;
            let size = 1.0 + layer * 1.8;
            let col = Color::new(0.7 + layer * 0.3, 0.8 + layer * 0.2, 1.0, 1.0);

            stars.push(Star {
                pos: Vec2::new(x, y),
                size,
                color: col,
                layer,
            });
        }

        Self {
            local_player_id: None,
            local_username: "Pilote".to_string(),
            connected: false,
            connecting: false,
            status_text: "Connexion au serveur...".to_string(),
            players: HashMap::new(),
            lasers: Vec::new(),
            minerals: Vec::new(),
            server_tick: 0,
            camera_pos: Vec2::ZERO,
            last_ping_send: 0.0,
            ping_ms: 0,
            particles: Vec::new(),
            float_texts: Vec::new(),
            stars,
        }
    }

    fn try_connect(&mut self, url: &str, token: &str) {
        self.connecting = true;
        self.status_text = format!("Connexion à {}...", url);

        let full_url = if token.is_empty() || token == "guest" {
            format!("{url}?token=guest")
        } else {
            format!("{url}?token={token}")
        };

        net::connect(&full_url);
        self.connected = true;
        self.connecting = false;
        self.status_text = "Connecté au serveur spatial".to_string();

        let auth_msg = ClientMessage::Auth {
            token: token.to_string(),
        };
        self.send_message(&auth_msg);
    }

    fn send_message(&mut self, msg: &ClientMessage) {
        if let Ok(bytes) = serialize_packet(msg) {
            net::send(&bytes);
        }
    }

    fn poll_network(&mut self) {
        while let Some(bytes) = net::try_recv() {
            if let Ok(server_msg) = deserialize_packet::<ServerMessage>(&bytes) {
                self.handle_server_message(server_msg);
            }
        }
    }

    fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::AuthSuccess {
                player_id,
                username,
                ship,
            } => {
                self.local_player_id = Some(player_id);
                self.local_username = username;
                self.camera_pos = ship.position;
                self.players.insert(player_id, ship);
                self.status_text = format!("Bienvenue, Pilote {}", self.local_username);
            }
            ServerMessage::AuthError { message } => {
                self.status_text = format!("Erreur auth: {}", message);
            }
            ServerMessage::WorldSnapshot(snapshot) => {
                self.server_tick = snapshot.tick;
                self.lasers = snapshot.lasers;
                self.minerals = snapshot.minerals;

                for server_ship in snapshot.players {
                    self.players.insert(server_ship.id, server_ship);
                }
            }
            ServerMessage::MineralCollected {
                player_id,
                mineral_type,
                value,
                ..
            } => {
                let player_pos = self.players.get(&player_id).map(|p| p.position);
                if let Some(pos) = player_pos {
                    let col_name = match mineral_type {
                        MineralType::Prometium => "Prometium",
                        MineralType::Endurium => "Endurium",
                        MineralType::Terbium => "Terbium",
                    };

                    let text_color = match mineral_type {
                        MineralType::Prometium => Color::new(1.0, 0.5, 0.2, 1.0),
                        MineralType::Endurium => Color::new(0.2, 0.8, 1.0, 1.0),
                        MineralType::Terbium => Color::new(0.3, 1.0, 0.4, 1.0),
                    };

                    self.spawn_float_text(
                        format!("+{} {} (+{} C.)", 1, col_name, value),
                        pos + Vec2::new(0.0, -30.0),
                        text_color,
                    );

                    self.spawn_explosion(pos, text_color, 12);
                }
            }
            ServerMessage::PlayerKilled {
                victim_id,
                killer_id: _,
            } => {
                let victim_pos = self.players.get(&victim_id).map(|p| p.position);
                if let Some(pos) = victim_pos {
                    self.spawn_explosion(pos, RED, 35);
                    self.spawn_explosion(pos, ORANGE, 25);
                }
            }
            ServerMessage::Pong { client_time, .. } => {
                let now = (macroquad::time::get_time() * 1000.0) as u64;
                if now >= client_time {
                    self.ping_ms = now - client_time;
                }
            }
            _ => {}
        }
    }

    fn spawn_float_text(&mut self, text: String, pos: Vec2, color: Color) {
        self.float_texts.push(FloatText {
            text,
            pos,
            color,
            lifetime: 1.5,
        });
    }

    fn spawn_explosion(&mut self, pos: Vec2, color: Color, count: usize) {
        for i in 0..count {
            let seed = (i as f32 + get_time() as f32) * 23.45;
            let angle = (seed.sin() * 43758.5453).fract() * std::f32::consts::TAU;
            let speed = 40.0 + ((seed + 1.0).sin() * 43758.5453).fract().abs() * 240.0;
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            let life = 0.5 + ((seed + 2.0).sin() * 43758.5453).fract().abs() * 0.7;
            let size = 2.0 + ((seed + 3.0).sin() * 43758.5453).fract().abs() * 3.0;

            self.particles.push(Particle {
                pos,
                vel,
                color,
                lifetime: life,
                max_lifetime: life,
                size,
            });
        }
    }

    fn update_fx(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.pos += p.vel * dt;
            p.vel = p.vel * 0.95;
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        for ft in &mut self.float_texts {
            ft.pos.y -= 25.0 * dt;
            ft.lifetime -= dt;
        }
        self.float_texts.retain(|ft| ft.lifetime > 0.0);
    }
}

#[macroquad::main("AstroBrawl")]
async fn main() {
    let mut game = GameClient::new();

    let server_url = "ws://localhost:3000/ws";
    let auth_token = "guest";
    game.try_connect(server_url, auth_token);

    loop {
        let dt = get_frame_time().min(0.05);

        game.poll_network();

        let now = get_time();
        if now - game.last_ping_send > 2.0 && game.connected {
            game.last_ping_send = now;
            let ping = ClientMessage::Ping {
                client_time: (now * 1000.0) as u64,
            };
            game.send_message(&ping);
        }

        let mut thrust = false;
        let mut target_angle = 0.0;
        let mut shoot = false;
        let mut respawn = false;

        if let Some(local_id) = game.local_player_id {
            if let Some(local_ship) = game.players.get(&local_id) {
                if local_ship.is_alive {
                    thrust = is_key_down(KeyCode::W)
                        || is_key_down(KeyCode::Z)
                        || is_key_down(KeyCode::Up)
                        || is_mouse_button_down(MouseButton::Right);

                    let (mx, my) = mouse_position();
                    let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
                    let mouse_dir = Vec2::new(mx - screen_center.x, my - screen_center.y);
                    target_angle = mouse_dir.y.atan2(mouse_dir.x);

                    shoot = is_mouse_button_pressed(MouseButton::Left)
                        || is_key_pressed(KeyCode::Space);

                    if thrust {
                        let back_dir =
                            Vec2::new(-local_ship.rotation.cos(), -local_ship.rotation.sin());
                        let exhaust_pos = local_ship.position + back_dir * (SHIP_RADIUS + 4.0);

                        game.particles.push(Particle {
                            pos: exhaust_pos,
                            vel: back_dir * 120.0 + local_ship.velocity * 0.2,
                            color: Color::new(0.0, 0.9, 1.0, 0.9),
                            lifetime: 0.35,
                            max_lifetime: 0.35,
                            size: 3.0,
                        });
                    }
                } else {
                    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::R) {
                        respawn = true;
                    }
                }

                game.camera_pos = game.camera_pos.lerp(local_ship.position, 0.15);
            }
        }

        if game.connected {
            let input_msg = ClientMessage::Input {
                thrust,
                target_angle,
            };
            game.send_message(&input_msg);

            if shoot {
                game.send_message(&ClientMessage::Shoot);
            }

            if respawn {
                game.send_message(&ClientMessage::Respawn);
            }
        }

        game.update_fx(dt);

        clear_background(Color::new(0.02, 0.03, 0.06, 1.0));

        let half_screen = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);

        for star in &game.stars {
            let screen_x = (star.pos.x - game.camera_pos.x * star.layer)
                .rem_euclid(screen_width() + 100.0)
                - 50.0;
            let screen_y = (star.pos.y - game.camera_pos.y * star.layer)
                .rem_euclid(screen_height() + 100.0)
                - 50.0;
            draw_circle(screen_x, screen_y, star.size, star.color);
        }

        let grid_size = 200.0;
        let start_x = ((game.camera_pos.x - half_screen.x) / grid_size).floor() * grid_size;
        let end_x = ((game.camera_pos.x + half_screen.x) / grid_size).ceil() * grid_size;
        let start_y = ((game.camera_pos.y - half_screen.y) / grid_size).floor() * grid_size;
        let end_y = ((game.camera_pos.y + half_screen.y) / grid_size).ceil() * grid_size;

        let grid_col = Color::new(0.0, 0.4, 0.6, 0.08);
        let mut cur_x = start_x;
        while cur_x <= end_x {
            let sx = cur_x - game.camera_pos.x + half_screen.x;
            draw_line(sx, 0.0, sx, screen_height(), 1.0, grid_col);
            cur_x += grid_size;
        }
        let mut cur_y = start_y;
        while cur_y <= end_y {
            let sy = cur_y - game.camera_pos.y + half_screen.y;
            draw_line(0.0, sy, screen_width(), sy, 1.0, grid_col);
            cur_y += grid_size;
        }

        let border_half_w = WORLD_WIDTH * 0.5;
        let border_half_h = WORLD_HEIGHT * 0.5;
        let b_tl_x = -border_half_w - game.camera_pos.x + half_screen.x;
        let b_tl_y = -border_half_h - game.camera_pos.y + half_screen.y;
        let b_w = WORLD_WIDTH;
        let b_h = WORLD_HEIGHT;

        let border_glow = ((get_time() * 3.0).sin() as f32 * 0.2 + 0.8).clamp(0.0, 1.0);
        let border_col = Color::new(1.0, 0.2, 0.3, 0.6 * border_glow);
        draw_rectangle_lines(b_tl_x, b_tl_y, b_w, b_h, 3.0, border_col);

        for mineral in &game.minerals {
            let sx = mineral.position.x - game.camera_pos.x + half_screen.x;
            let sy = mineral.position.y - game.camera_pos.y + half_screen.y;

            if sx >= -40.0
                && sx <= screen_width() + 40.0
                && sy >= -40.0
                && sy <= screen_height() + 40.0
            {
                let rgba = mineral.mineral_type.color_rgba();
                let ore_col = Color::new(rgba[0], rgba[1], rgba[2], rgba[3]);

                let pulse = (get_time() * 4.0 + (mineral.id as f64)).sin() as f32 * 0.25 + 0.75;
                let aura_col = Color::new(rgba[0], rgba[1], rgba[2], 0.25 * pulse);
                draw_circle(sx, sy, MINERAL_RADIUS * 1.6, aura_col);

                let r = MINERAL_RADIUS;
                let v1 = to_mq(Vec2::new(sx, sy - r));
                let v2 = to_mq(Vec2::new(sx + r, sy));
                let v3 = to_mq(Vec2::new(sx, sy + r));
                let v4 = to_mq(Vec2::new(sx - r, sy));

                draw_triangle(v1, v2, v3, ore_col);
                draw_triangle(v1, v3, v4, ore_col);
                draw_triangle_lines(v1, v2, v3, 1.5, WHITE);
                draw_triangle_lines(v1, v3, v4, 1.5, WHITE);
            }
        }

        for laser in &game.lasers {
            let sx = laser.position.x - game.camera_pos.x + half_screen.x;
            let sy = laser.position.y - game.camera_pos.y + half_screen.y;

            let dir = laser.velocity.normalize();
            let tail = Vec2::new(sx, sy) - dir * 18.0;
            let head = Vec2::new(sx, sy) + dir * 6.0;

            draw_line(
                tail.x,
                tail.y,
                head.x,
                head.y,
                6.0,
                Color::new(0.0, 0.9, 1.0, 0.4),
            );
            draw_line(tail.x, tail.y, head.x, head.y, 2.5, WHITE);
        }

        for p in &game.particles {
            let sx = p.pos.x - game.camera_pos.x + half_screen.x;
            let sy = p.pos.y - game.camera_pos.y + half_screen.y;
            let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut col = p.color;
            col.a *= alpha;
            draw_circle(sx, sy, p.size * alpha, col);
        }

        for player in game.players.values() {
            if !player.is_alive {
                continue;
            }

            let sx = player.position.x - game.camera_pos.x + half_screen.x;
            let sy = player.position.y - game.camera_pos.y + half_screen.y;

            let is_local = Some(player.id) == game.local_player_id;

            let rot = player.rotation;
            let nose = to_mq(Vec2::new(sx + rot.cos() * 24.0, sy + rot.sin() * 24.0));
            let left_wing = to_mq(Vec2::new(
                sx + (rot + 2.5).cos() * 20.0,
                sy + (rot + 2.5).sin() * 20.0,
            ));
            let right_wing = to_mq(Vec2::new(
                sx + (rot - 2.5).cos() * 20.0,
                sy + (rot - 2.5).sin() * 20.0,
            ));
            let engine = to_mq(Vec2::new(
                sx + (rot + std::f32::consts::PI).cos() * 14.0,
                sy + (rot + std::f32::consts::PI).sin() * 14.0,
            ));

            if player.is_thrusting {
                let flame_len = 16.0 + ((get_time() * 30.0).sin() as f32 * 4.0);
                let flame_tip = to_mq(Vec2::new(
                    sx + (rot + std::f32::consts::PI).cos() * (14.0 + flame_len),
                    sy + (rot + std::f32::consts::PI).sin() * (14.0 + flame_len),
                ));
                draw_triangle(left_wing * 0.4 + engine * 0.6, flame_tip, right_wing * 0.4 + engine * 0.6, ORANGE);
            }

            let hull_col = if is_local {
                Color::new(0.05, 0.45, 0.75, 1.0)
            } else {
                Color::new(0.65, 0.15, 0.2, 1.0)
            };
            draw_triangle(nose, left_wing, engine, hull_col);
            draw_triangle(nose, right_wing, engine, hull_col);
            draw_triangle_lines(
                nose,
                left_wing,
                engine,
                2.0,
                if is_local { SKYBLUE } else { RED },
            );
            draw_triangle_lines(
                nose,
                right_wing,
                engine,
                2.0,
                if is_local { SKYBLUE } else { RED },
            );

            let cockpit = Vec2::new(sx + rot.cos() * 6.0, sy + rot.sin() * 6.0);
            draw_circle(cockpit.x, cockpit.y, 4.0, Color::new(0.2, 0.9, 1.0, 0.9));

            let name_width = measure_text(&player.username, None, 14, 1.0).width;
            draw_text(
                &player.username,
                sx - name_width * 0.5,
                sy - 34.0,
                14.0,
                if is_local { GOLD } else { WHITE },
            );

            let bar_w = 44.0;
            let bar_h = 4.0;
            let bar_x = sx - bar_w * 0.5;

            let shield_pct = (player.shield / player.max_shield).clamp(0.0, 1.0);
            draw_rectangle(bar_x, sy - 28.0, bar_w, bar_h, Color::new(0.05, 0.1, 0.2, 0.8));
            draw_rectangle(
                bar_x,
                sy - 28.0,
                bar_w * shield_pct,
                bar_h,
                Color::new(0.0, 0.85, 1.0, 1.0),
            );

            let hp_pct = (player.health / player.max_health).clamp(0.0, 1.0);
            draw_rectangle(bar_x, sy - 22.0, bar_w, bar_h, Color::new(0.2, 0.05, 0.05, 0.8));
            draw_rectangle(
                bar_x,
                sy - 22.0,
                bar_w * hp_pct,
                bar_h,
                Color::new(0.1, 0.95, 0.3, 1.0),
            );
        }

        for ft in &game.float_texts {
            let sx = ft.pos.x - game.camera_pos.x + half_screen.x;
            let sy = ft.pos.y - game.camera_pos.y + half_screen.y;
            let mut col = ft.color;
            col.a = (ft.lifetime / 1.5).clamp(0.0, 1.0);
            draw_text(&ft.text, sx, sy, 16.0, col);
        }

        let radar_size = 140.0;
        let radar_pad = 20.0;
        let rx = screen_width() - radar_size - radar_pad;
        let ry = screen_height() - radar_size - radar_pad;

        draw_rectangle(rx, ry, radar_size, radar_size, Color::new(0.04, 0.07, 0.14, 0.85));
        draw_rectangle_lines(rx, ry, radar_size, radar_size, 1.5, Color::new(0.0, 0.9, 1.0, 0.4));
        draw_line(
            rx + radar_size * 0.5,
            ry,
            rx + radar_size * 0.5,
            ry + radar_size,
            1.0,
            Color::new(0.0, 0.9, 1.0, 0.15),
        );
        draw_line(
            rx,
            ry + radar_size * 0.5,
            rx + radar_size,
            ry + radar_size * 0.5,
            1.0,
            Color::new(0.0, 0.9, 1.0, 0.15),
        );

        let scale = radar_size / WORLD_WIDTH;
        let radar_center = Vec2::new(rx + radar_size * 0.5, ry + radar_size * 0.5);

        for m in &game.minerals {
            let mx = radar_center.x + m.position.x * scale;
            let my = radar_center.y + m.position.y * scale;
            if mx >= rx && mx <= rx + radar_size && my >= ry && my <= ry + radar_size {
                draw_circle(mx, my, 1.2, Color::new(0.2, 0.9, 1.0, 0.7));
            }
        }

        for p in game.players.values() {
            if !p.is_alive {
                continue;
            }
            let px = radar_center.x + p.position.x * scale;
            let py = radar_center.y + p.position.y * scale;
            let is_local = Some(p.id) == game.local_player_id;
            let p_col = if is_local { GREEN } else { RED };
            draw_circle(px, py, if is_local { 3.0 } else { 2.2 }, p_col);
        }

        let sweep_angle = (get_time() * 2.0) as f32;
        let sweep_dir = Vec2::new(sweep_angle.cos(), sweep_angle.sin()) * (radar_size * 0.5);
        draw_line(
            radar_center.x,
            radar_center.y,
            radar_center.x + sweep_dir.x,
            radar_center.y + sweep_dir.y,
            1.0,
            Color::new(0.0, 1.0, 0.8, 0.3),
        );

        let status_color = if game.connected {
            Color::new(0.0, 1.0, 0.6, 1.0)
        } else {
            Color::new(1.0, 0.4, 0.3, 1.0)
        };
        draw_circle(30.0, screen_height() - 25.0, 5.0, status_color);
        draw_text(&game.status_text, 44.0, screen_height() - 20.0, 13.0, WHITE);
        if game.connected {
            draw_text(
                &format!("Ping: {} ms", game.ping_ms),
                44.0,
                screen_height() - 38.0,
                12.0,
                GRAY,
            );
        }

        if let Some(local_id) = game.local_player_id {
            if let Some(local_ship) = game.players.get(&local_id) {
                if !local_ship.is_alive {
                    draw_rectangle(
                        0.0,
                        screen_height() * 0.35,
                        screen_width(),
                        120.0,
                        Color::new(0.05, 0.05, 0.1, 0.88),
                    );
                    let death_title = "⚡ VAISSEAU DÉTRUIT ⚡";
                    let t_w = measure_text(death_title, None, 32, 1.0).width;
                    draw_text(
                        death_title,
                        (screen_width() - t_w) * 0.5,
                        screen_height() * 0.43,
                        32.0,
                        RED,
                    );

                    let respawn_sub = "Appuyez sur ESPACE ou R pour réparer et réapparaître";
                    let s_w = measure_text(respawn_sub, None, 18, 1.0).width;
                    draw_text(
                        respawn_sub,
                        (screen_width() - s_w) * 0.5,
                        screen_height() * 0.48,
                        18.0,
                        WHITE,
                    );
                }
            }
        }

        next_frame().await;
    }
}
