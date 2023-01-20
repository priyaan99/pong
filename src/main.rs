use std::f32::consts::PI;
use std::ops::Mul;

use macroquad::{prelude::*, audio::Sound};
use macroquad::audio::{self, play_sound_once};

fn window_conf() -> Conf {
    Conf {
        window_title: "Pong".to_owned(),
        window_width: 512,
        window_height: 320,
        fullscreen: false,
        ..Default::default()
    }
}

struct State {
    puck: Puck,
    left: Paddle,
    right: Paddle,
    out: bool,
    win: Option<Side>,
    sound_system: SoundState,
}

impl State {
    const MAX_SCORE: i32 = 5;

    async fn init() -> Self {
        let puck = Puck::init();
        const OFFSET: f32 = 10.0;
        let left = Paddle::init(Side::Left, OFFSET);
        let right= Paddle::init(Side::Right, OFFSET);

        let out = true;

        let mut sound_system = SoundState::new();
        sound_system.load_sound().await;

        Self { puck, left, right, out, sound_system, win: None}
    }

    async fn reset(&mut self) {
        *self = Self::init().await;
        self.out = false;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = State::init().await;

    loop {
        if !state.out {
            if is_key_down(KeyCode::Up) { state.right.move_up(); }
            if is_key_down(KeyCode::Down) { state.right.move_down(); }

            if is_key_down(KeyCode::W) { state.left.move_up(); }
            if is_key_down(KeyCode::S) { state.left.move_down(); }

            let puck_circle = Circle::from(&state.puck); 
            let left_rect = Rect::from(&state.left);
            let right_rect = Rect::from(&state.right);

            if puck_circle.overlaps_rect(&right_rect) {

                state.puck.set_y_angle(calculate_angle(&puck_circle, &right_rect), &state.right.side);
                state.sound_system.play_sound_one();

            } else if puck_circle.overlaps_rect(&left_rect) {

                state.puck.set_y_angle(calculate_angle(&puck_circle, &left_rect), &state.left.side);
                state.sound_system.play_sound_one();

            } else {
                if state.puck.x() > screen_width() {
                    state.left.score += 1;
                    state.puck.reset();
                }

                if state.puck.x() < 0.0 {
                    state.right.score += 1;
                    state.puck.reset();
                }
            }

            state.puck.update();
            state.left.update();
            state.right.update();

            if state.left.score() > State::MAX_SCORE / 2 {
                state.win = Some(Side::Left);
                state.out = true;
            }

            if state.right.score() > State::MAX_SCORE / 2 {
                state.win = Some(Side::Right);
                state.out = true;
            }
        } 

        if state.out {
            if is_key_pressed(KeyCode::Enter) {
                println!("asdf");
                state.reset().await;
            }
        }

        clear_background(WHITE);

        if !state.out {
            state.puck.draw();
            state.left.draw();
            state.right.draw();
        } 

        if state.out {
            const FONT_SIZE: u16 = 20; 
            let message = "Press Enter To Play"; 
            let message_dimension = measure_text(message, None, FONT_SIZE, 1.0);
            
            draw_text(
                message, 
                screen_width() / 2. - message_dimension.width / 2., 
                screen_height() / 2. - message_dimension.height / 2., 
                FONT_SIZE as f32, 
                RED
            );

            const RESULT_FONT_SIZE: u16 = 30; 
            let mut result_pos = vec2(screen_width() / 2., screen_height() * 1./3.);
            let mut result = "";

            if let Some(side) = state.win.clone() {
                match side {
                    Side::Left => result = "Left Side Won",
                    Side::Right=> result = "Right Side Won",
                }
            } else {
                result = "Lets Play"
            }

            let result_dimension = measure_text(result, None, RESULT_FONT_SIZE, 1.0);
            result_pos.x -= result_dimension.width / 2.;

            draw_text(
                result, 
                result_pos.x, 
                result_pos.y, 
                RESULT_FONT_SIZE as f32, 
                RED
            )
        }

        next_frame().await
    }
}

fn calculate_angle(puck: &Circle, paddle: &Rect) -> f32 {
    let dis = puck.y - paddle.y;
    let mut angle = (dis/paddle.h) * 180.0;
    angle = 180.0 - angle; // flip angle relative to x axis
    angle = angle.clamp(20.0, 160.0);

    println!("{}", angle);
    angle
}

struct SoundState {
    sound_one: Option<Sound>,
}

impl SoundState {
    fn new() -> Self {
        Self {
            sound_one: None
        }
    } 

    async fn load_sound(&mut self) {
        self.sound_one = Some(audio::load_sound("forceField.ogg").await.unwrap());
    }

    fn play_sound_one(&self) {
        if let Some(sound) = self.sound_one {
            play_sound_once(sound);
        }
    }
}


struct Puck {
    radius: f32,
    pos: Vec2,
    speed: Vec2,
    dir: Vec2,
}

impl From<&Puck> for Circle {
    fn from(value: &Puck) -> Self {
        Circle { 
            x: value.pos.x, 
            y: value.pos.y, 
            r: value.radius 
        }
    }
}

impl Puck {
    const SPEED: f32 = 200.0;

    fn init() -> Self {
        Puck {
            radius: 10.0,
            pos: Vec2::new(screen_width()/2.0, screen_height()/2.0),
            speed: Vec2::new(Self::SPEED, Self::SPEED),
            dir: Vec2::from_angle(PI),
        }
    }

    fn x(&self) -> f32 { self.pos.x }
    // fn y(&self) -> f32 { self.pos.y }
    // fn radius(&self) -> f32 { self.radius }

    fn reset(&mut self) {
        *self = Self::init();
    }

    fn set_y_angle(&mut self, mut angle: f32, side: &Side) {
        match side {
            Side::Left => { 
                angle = 90.0 - angle; 
            },
            Side::Right => angle += 90.0,
        };
        
        self.dir = Vec2::from_angle(angle.to_radians());
    }

    fn update(&mut self) {
        let velocity = self.dir.mul(self.speed);
        self.pos = self.pos + velocity * get_frame_time();

        // collision: puck vs wall (top and bottom)
        if (self.pos.y < 0.0+self.radius) || (self.pos.y > screen_height() - self.radius) {
            self.speed.y *= -1.0;
        }
    }

    fn draw(&self) {
        draw_circle(self.pos.x, self.pos.y, self.radius, RED);
    }
}

struct Paddle {
    pos: Vec2,
    size: Vec2,
    side: Side,
    score: i32,
}

impl From<&Paddle> for Rect {
    fn from(value: &Paddle) -> Self {
        Rect { 
            x: value.pos.x, 
            y: value.pos.y, 
            w: value.size.x, 
            h: value.size.y 
        }
    }
}

#[derive(Clone)]
enum Side {
    Left,
    Right,
}

impl Paddle {
    const SPEED: f32 = 200.0;

    fn init(side: Side, offset: f32) -> Self {
        let size = Vec2::new(10.0, 50.0);

        let x = match side {
            Side::Left => offset,
            Side::Right => screen_width() - size.x - offset,
        };

        let pos  = Vec2::new(x, screen_height()/2.0 - size.y/2.0);

        Self { size, pos, side, score: 0 }
    }

    // fn height(&self) -> f32 { self.size.y }
    // fn width(&self) -> f32 { self.size.x }
    // fn x(&self) -> f32 {self.pos.x }
    // fn y(&self) -> f32 {self.pos.y }
    fn score(&self) -> i32 { self.score }

    fn move_up(&mut self) {
        self.pos.y -= Self::SPEED * get_frame_time()
    }

    fn move_down(&mut self) {
        self.pos.y += Self::SPEED * get_frame_time()
    }

    fn update(&mut self) {
        self.pos.y = self.pos.y.clamp(0.0, screen_height() - self.size.y);
    }

    fn draw(&self) {
        draw_rectangle(self.pos.x, self.pos.y, self.size.x, self.size.y, RED);
    }
}
