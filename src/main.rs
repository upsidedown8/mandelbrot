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

const MAX_ITER_INCREMENT: usize = 32;

const COLORS: [[u8; 3]; 16] = [
    [66, 30, 15],
    [25, 7, 26],
    [9, 1, 47],
    [4, 4, 73],
    [0, 7, 100],
    [12, 44, 138],
    [24, 82, 177],
    [57, 125, 209],
    [134, 181, 229],
    [211, 236, 248],
    [241, 233, 191],
    [248, 201, 95],
    [255, 170, 0],
    [204, 128, 0],
    [153, 87, 0],
    [106, 52, 3],
];

struct GameState {
    init_iterations: usize,
    current_iterations: usize,

    max_iterations: usize,

    buffer: Vec<u8>,

    iteration_counts: Vec<usize>,
    iteration_vals: Vec<[f64; 4]>,

    x_min: f64,
    x_range: f64,

    y_min: f64,
    y_range: f64,

    redraw: bool,
}

impl GameState {
    pub fn new(_ctx: &mut Context) -> Self {
        Self {
            init_iterations: 32,
            max_iterations: 64,
            current_iterations: 0,
            buffer: vec![0; 4 * SCREEN_WIDTH * SCREEN_HEIGHT],
            iteration_counts: vec![0; SCREEN_HEIGHT * SCREEN_WIDTH],
            iteration_vals: vec![[0.0; 4]; SCREEN_HEIGHT * SCREEN_WIDTH],
            x_min: -2.5,
            x_range: 3.5,
            y_min: -1.0,
            y_range: 2.0,
            redraw: true,
        }
    }
}

impl GameState {
    fn handle_input(&mut self, ctx: &mut Context) {
        // iterations
        if keyboard::is_key_pressed(ctx, KeyCode::E) {
            self.redraw = false;
            self.max_iterations += 64;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Q) && self.max_iterations >= 64 {
            self.redraw = true;
            self.max_iterations -= 64;
        }

        // reset
        if keyboard::is_key_pressed(ctx, KeyCode::R) {
            self.redraw = true;
            self.x_min = -2.5;
            self.x_range = 3.5;
            self.y_min = -1.0;
            self.y_range = 2.0;
        }

        // movement
        if keyboard::is_key_pressed(ctx, KeyCode::Left) || keyboard::is_key_pressed(ctx, KeyCode::A)
        {
            self.redraw = true;
            self.x_min -= MOVEMENT_MULTIPLIER * self.x_range;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Right)
            || keyboard::is_key_pressed(ctx, KeyCode::D)
        {
            self.redraw = true;
            self.x_min += MOVEMENT_MULTIPLIER * self.x_range;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Up) || keyboard::is_key_pressed(ctx, KeyCode::W) {
            self.redraw = true;
            self.y_min -= MOVEMENT_MULTIPLIER * self.y_range;
        }
        if keyboard::is_key_pressed(ctx, KeyCode::Down) || keyboard::is_key_pressed(ctx, KeyCode::S)
        {
            self.redraw = true;
            self.y_min += MOVEMENT_MULTIPLIER * self.y_range;
        }

        // zoom out
        if keyboard::is_key_pressed(ctx, KeyCode::Minus) {
            self.redraw = true;
            self.x_min -= self.x_range * 0.5 * (ZOOM_MULTIPLIER - 1.0);
            self.y_min -= self.y_range * 0.5 * (ZOOM_MULTIPLIER - 1.0);

            self.x_range *= ZOOM_MULTIPLIER;
            self.y_range *= ZOOM_MULTIPLIER;
        }
        // zoom in
        if keyboard::is_key_pressed(ctx, KeyCode::Equals) {
            self.redraw = true;
            self.x_range /= ZOOM_MULTIPLIER;
            self.y_range /= ZOOM_MULTIPLIER;

            self.x_min += self.x_range * 0.5 * (ZOOM_MULTIPLIER - 1.0);
            self.y_min += self.y_range * 0.5 * (ZOOM_MULTIPLIER - 1.0);
        }
    }
    fn get_xy(&self, s_x: usize, s_y: usize, w: f64, h: f64) -> (f64, f64) {
        (
            self.x_min + self.x_range * (s_x as f64 / w),
            self.y_min + self.y_range * (s_y as f64 / h),
        )
    }
    fn get_color(&self, val: f64) -> [u8; 4] {
        let mut pos = (COLORS.len() - 1) as f64 * val;

        let color1 = pos.floor() as usize;
        let color2 = color1 + 1;

        // 0 <= pos <= 1
        pos -= color1 as f64;

        let mut result = [0; 4];

        for i in 0..3 {
            let range = COLORS[color2][i] as f64 - COLORS[color1][i] as f64;
            result[i] = COLORS[color1][i] + (range * pos) as u8;
        }

        result[3] = 255;

        result
    }
    fn calculate_mandelbrot(&mut self, max_iter: usize) {
        let w = SCREEN_WIDTH as f64;
        let h = SCREEN_HEIGHT as f64;

        self.current_iterations = max_iter;

        for s_x in 0..SCREEN_WIDTH {
            for s_y in 0..SCREEN_HEIGHT {
                let pos = s_x + s_y * SCREEN_WIDTH;

                let (x0, y0) = self.get_xy(s_x, s_y, w, h);

                // load values
                let mut x = self.iteration_vals[pos][0];
                let mut y = self.iteration_vals[pos][1];
                let mut x_sq = self.iteration_vals[pos][2];
                let mut y_sq = self.iteration_vals[pos][3];
                let mut iter = self.iteration_counts[pos];

                while x_sq + y_sq <= 4.0 && iter < max_iter {
                    y = 2.0 * x * y + y0;
                    x = x_sq - y_sq + x0;

                    x_sq = x * x;
                    y_sq = y * y;

                    iter += 1;
                }

                // save values
                self.iteration_vals[pos][0] = x;
                self.iteration_vals[pos][1] = y;
                self.iteration_vals[pos][2] = x_sq;
                self.iteration_vals[pos][3] = y_sq;
                self.iteration_counts[pos] = iter;

                // set color
                let color = self.get_color(1.0 - iter as f64 / max_iter as f64);
                self.buffer[4 * pos..4 * (pos + 1)].copy_from_slice(&color);
            }
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.handle_input(ctx);

        if self.redraw {
            // reset
            self.current_iterations = 0;
            self.iteration_counts.fill(0);
            self.iteration_vals.fill([0.0; 4]);
            self.calculate_mandelbrot(self.init_iterations);
        } else if self.current_iterations <= self.max_iterations {
            println!("{}", self.current_iterations);
            self.calculate_mandelbrot(
                self.current_iterations + self.current_iterations.min(MAX_ITER_INCREMENT),
            );
        }

        self.redraw = false;

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
