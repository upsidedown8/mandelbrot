use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler, KeyCode},
    graphics::{self, Image},
    input::keyboard,
    Context, ContextBuilder, GameResult,
};

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 450;

const MOVEMENT_MULTIPLIER: f64 = 0.025;
const ZOOM_MULTIPLIER: f64 = 1.25;

struct GameState {
    n_iter: usize,

    buffer: Vec<u8>,
    iteration_counts: Vec<usize>,

    x_min: f64,
    x_range: f64,

    y_min: f64,
    y_range: f64,

    changed: bool,
}

impl GameState {
    pub fn new(_ctx: &mut Context) -> Self {
        Self {
            n_iter: 250,
            buffer: vec![0; 4 * SCREEN_WIDTH * SCREEN_HEIGHT],
            iteration_counts: vec![0; SCREEN_HEIGHT * SCREEN_WIDTH],
            x_min: -2.5,
            x_range: 3.5,
            y_min: -1.0,
            y_range: 2.0,
            changed: true,
        }
    }
}

fn interpolate(min: f64, range: f64, prop: f64) -> f64 {
    min + range * prop
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // let time_elapsed = ggez::timer::delta(ctx).as_secs_f32();

        // movement
        if keyboard::is_key_pressed(ctx, KeyCode::Left) || keyboard::is_key_pressed(ctx, KeyCode::A)
        {
            self.x_min -= MOVEMENT_MULTIPLIER * self.x_range;
            self.changed = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Right)
            || keyboard::is_key_pressed(ctx, KeyCode::D)
        {
            self.x_min += MOVEMENT_MULTIPLIER * self.x_range;
            self.changed = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) || keyboard::is_key_pressed(ctx, KeyCode::W) {
            self.y_min -= MOVEMENT_MULTIPLIER * self.y_range;
            self.changed = true;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) || keyboard::is_key_pressed(ctx, KeyCode::S)
        {
            self.y_min += MOVEMENT_MULTIPLIER * self.y_range;
            self.changed = true;
        }

        // zoom out
        if keyboard::is_key_pressed(ctx, KeyCode::Comma) {
            self.x_range *= ZOOM_MULTIPLIER;
            self.y_range *= ZOOM_MULTIPLIER;
            self.changed = true;
        }
        // zoom in
        if keyboard::is_key_pressed(ctx, KeyCode::Period) {
            self.x_range /= ZOOM_MULTIPLIER;
            self.y_range /= ZOOM_MULTIPLIER;
            self.changed = true;
        }

        if self.changed {
            let width = SCREEN_WIDTH as f64;
            let height = SCREEN_HEIGHT as f64;

            self.iteration_counts.fill(0);

            for s_x in 0..SCREEN_WIDTH {
                for s_y in 0..SCREEN_HEIGHT {
                    let x0 = interpolate(self.x_min, self.x_range, s_x as f64 / width);
                    let y0 = interpolate(self.y_min, self.y_range, s_y as f64 / height);

                    let mut x = 0.0;
                    let mut y = 0.0;

                    let pos = s_x + s_y * SCREEN_WIDTH;

                    while x * x + y * y <= 4.0 && self.iteration_counts[pos] < self.n_iter {
                        let tmp = x * x - y * y + x0;
                        y = 2.0 * x * y + y0;
                        x = tmp;

                        self.iteration_counts[pos] += 1;
                    }
                }
            }

            for x in 0..SCREEN_WIDTH {
                for y in 0..SCREEN_HEIGHT {
                    let pos = x + y * SCREEN_WIDTH;

                    let prop = self.iteration_counts[pos] as f64 / self.n_iter as f64;
                    let shade = 255 - (255.0 * prop) as u8;

                    let start = 4 * (x + y * SCREEN_WIDTH);
                    self.buffer[start..start + 4].copy_from_slice(&[0, 0, shade, 255]);
                }
            }
        }

        self.changed = false;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let img = Image::from_rgba8(ctx, SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16, &self.buffer)?;
        graphics::draw(ctx, &img, graphics::DrawParam::default())?;
        graphics::present(ctx)
    }
}

fn main() -> GameResult {
    let (mut ctx, mut event_loop) = ContextBuilder::new("mandelbrot", "Tom Thorogood")
        .window_mode(WindowMode::default().dimensions(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32))
        .window_setup(WindowSetup::default().title("Mandelbrot set"))
        .build()?;
    let mut game = GameState::new(&mut ctx);
    event::run(&mut ctx, &mut event_loop, &mut game)
}
