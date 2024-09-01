use bracket_lib::prelude::*;
use std::cmp;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;
const SCORE_PER_OBSTACLE: f32 = 0.05;
const SPRITE_COORD_TO_CONSOLE_COORD: i32 = 8;

struct State {
    player: Player,
    frame_time: f32,
    frame: usize,
    obstacles: Vec<Obstacle>,
    mode: GameMode,
    score: f32,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            frame: 0,
            obstacles: vec![Obstacle::new(SCREEN_WIDTH, SCREEN_HEIGHT / 2, 0.0)],
            mode: GameMode::Menu,
            score: 0.0,
        }
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P)lay Game");
        ctx.print_centered(9, "(Q)uit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.obstacles = vec![Obstacle::new(SCREEN_WIDTH, SCREEN_HEIGHT / 2, 0.0)];
        self.mode = GameMode::Playing;
        self.score = 0.0;
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.frame = self.frame.wrapping_add(1);

            self.player.gravity_and_move();
            let last_gap_y = self.obstacles.last().unwrap().gap_y;
            let mut random = RandomNumberGenerator::new();
            let mut obstacle_center = cmp::min(
                random.range(last_gap_y - 7, last_gap_y + 7),
                SCREEN_HEIGHT - 20,
            );
            obstacle_center = cmp::max(obstacle_center, 20);

            self.obstacles.push(Obstacle::new(
                self.player.x + SCREEN_WIDTH,
                obstacle_center,
                self.score,
            ));
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx, self.frame);

        if self.player.x > self.obstacles[0].x {
            self.score += SCORE_PER_OBSTACLE;
            self.obstacles.remove(0);
        }

        for obstacle in self.obstacles.iter_mut() {
            obstacle.render(ctx, self.player.x);

            if self.player.y > SCREEN_HEIGHT || obstacle.hit_obstacle(&self.player) {
                self.mode = GameMode::End;
            }
        }

        ctx.print_color(0, 0, WHITE, BLACK, "Press SPACE to flap.");
        ctx.print_color(0, 1, WHITE, BLACK, &format!("Score: {}", self.score as i32));
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score as i32));
        ctx.print_centered(8, "(P)lay Game");
        ctx.print_centered(9, "(Q)uit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

enum GameMode {
    Menu,
    Playing,
    End,
}

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm, frame: usize) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.add_sprite(
            Rect::with_size(
                0,
                (self.y * SPRITE_COORD_TO_CONSOLE_COORD) - SPRITE_COORD_TO_CONSOLE_COORD,
                32,
                32,
            ),
            400 - self.y,
            RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
            frame % 4,
        );
        ctx.set_active_console(0);
        // use to display actual position
        // ctx.set(0, self.y, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        self.y += self.velocity as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, last_gap: i32, score: f32) -> Self {
        let mut random = RandomNumberGenerator::new();
        let center = random.range(last_gap - 5, last_gap + 5);
        Obstacle {
            x,
            gap_y: center,
            size: i32::max(5, 30 - (score as i32)),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;
        ctx.set_active_console(0);

        ctx.set_active_console(1);
        for y in 0..self.gap_y - half_size {
            ctx.add_sprite(
                Rect::with_size(
                    screen_x * SPRITE_COORD_TO_CONSOLE_COORD,
                    y * SPRITE_COORD_TO_CONSOLE_COORD,
                    SPRITE_COORD_TO_CONSOLE_COORD,
                    SPRITE_COORD_TO_CONSOLE_COORD,
                ),
                400 - y,
                RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                4,
            );
            //ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }

        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.add_sprite(
                Rect::with_size(
                    screen_x * SPRITE_COORD_TO_CONSOLE_COORD,
                    y * SPRITE_COORD_TO_CONSOLE_COORD,
                    SPRITE_COORD_TO_CONSOLE_COORD,
                    SPRITE_COORD_TO_CONSOLE_COORD,
                ),
                400 - y,
                RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                4,
            );
            //ctx.set(screen_x, y, RED, BLACK, to_cp437('|'));
        }
        ctx.set_active_console(0);
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;

        let does_x_match = player.x == self.x - SPRITE_COORD_TO_CONSOLE_COORD / 2;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;

        does_x_match && (player_above_gap || player_below_gap)
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .with_sprite_console(640, 400, 0)
        .with_sprite_sheet(
            SpriteSheet::new("resources/all.png")
                .add_sprite(Rect::with_size(0, 0, 939, 678))
                .add_sprite(Rect::with_size(939, 0, 939, 678))
                .add_sprite(Rect::with_size(1878, 0, 939, 678))
                .add_sprite(Rect::with_size(2817, 0, 939, 678))
                .add_sprite(Rect::with_size(3756, 0, 256, 256)),
        )
        .build()?;

    main_loop(context, State::new())
}
