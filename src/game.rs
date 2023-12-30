use crate::utils::log;
use crate::utils::random;
use crate::textdisplay::update_text_display;
use crate::textdisplay::set_background_colour;
use crate::highscore;

use wasm_bindgen::prelude::*;


pub const DEFAULT_INPUT: char = '1';
pub const INPUT_DOWN: char = 's';
const INITIAL_SNAKE_LEN: usize = 3;
const FRAME_RATE_SPEED_1: i32 = 1000 / 10;
const SPEED_INCREASE_MS_MODE_KEYBOARD: i32 = 10;
const SPEED_INCREASE_MS_MODE_TOUCH: i32 = 5;
const SPEED_INCREASE_AT_SCORE: u32 = 3;
const SPEED_TO_SET_BG_COL: i32 = 2;
const TOUCH_MODE_FOOD_BORDER_OFFSET: u32 = 5;

const COLOURS: [&str; 21] = [
    "#050",
    "#0A0",
    "#0F0",
    "#00F",
    "#00A",
    "#005",
    "#055",
    "#0AA",
    "#0FF",
    "#F0F",
    "#A0A",
    "#505",
    "#500",
    "#A00",
    "#F00",
    "#FA0",
    "#A50",
    "#A30",
    "#550",
    "#AA0",
    "#FF0",
];

const BACKGROUND_COLOURS: [&str; 7] = [
    "#00F",
    "#0FF",
    "#F0F",
    "#F00",
    "#FA0",
    "#FF0",
    "#0F0",
];

#[derive(Copy, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(PartialEq)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}


#[derive(PartialEq)]
pub enum GameMode {
    FAST,
    LONG,
}


pub struct Game {
    width: u32,
    height: u32,
    block_size: u32,
    draw_grid: bool,
    speed: i32,
    score: u32,
    context: Option<web_sys::CanvasRenderingContext2d>,
    snake: Vec<Point>,
    direction: Direction,
    food: Point,
    over: bool,
    pause: bool,
    input: char,
    pressed: bool,
    timestamp_last_frame: u32,
    colour_index: usize,
    touch_mode: bool,
    game_mode: GameMode,
    name: String,
}


impl Game {
    pub const fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            block_size: 1,
            draw_grid: false,
            speed: 0,
            score: 0,
            context: None,
            snake: Vec::new(),
            direction: Direction::DOWN,
            food: Point{x:0,y:0},
            over: true,
            pause: false,
            input: DEFAULT_INPUT,
            pressed: false,
            timestamp_last_frame: 0,
            colour_index: 0,
            touch_mode: false,
            game_mode: GameMode::FAST,
            name: String::new(),
       }
    }
    pub fn set_state(
        &mut self, 
        width: u32, 
        height: u32, 
        block_size: u32, 
        draw_grid: bool, 
        touch_mode: bool,
        game_mode: GameMode,
        name: &str,
        context: web_sys::CanvasRenderingContext2d)
    {
        log!("  re-setting game state! width: {}, height: {}, block_size: {}, draw_grid: {}, touch_mode: {}", 
            width, height, block_size, draw_grid, touch_mode);

        self.width = width;
        self.height = height;
        self.block_size = block_size;
        self.draw_grid = draw_grid;
        self.touch_mode = touch_mode;
        self.game_mode = game_mode;
        self.name = name.to_string();
        self.speed = 1;
        self.score = 0;
        self.context = Some(context);
        self.snake = self.init_snake();
        self.direction = Direction::DOWN;
        self.over = false;
        self.pause = false;
        self.input = INPUT_DOWN;
        self.pressed = false;
        self.timestamp_last_frame = 0;
        self.colour_index = 0;
        set_background_colour("#FFF");
        update_text_display(self.score, self.speed as u32);
        self.place_food();
    }

    fn init_snake(&self) -> Vec<Point> {
        let first_point = self.calc_center();
        let mut snake = vec![first_point];
        let mut i :i32 = 1;
        while i < INITIAL_SNAKE_LEN.try_into().unwrap() {
            let point = Point{x: first_point.x, y: first_point.y - i};
            snake.push(point);
            i = i + 1;
        }
        snake
    }

    pub fn is_over(&self) -> bool {
        self.over
    }

    pub fn set_input(&mut self, input: char) {
        self.input = input;
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    pub fn world_loop_contents(&mut self, timestamp :u32) -> bool {
        if !self.over && self.enough_time_passed(timestamp) {
            self.process_input();
            if self.pause {
                self.draw_pause();
            } else {
                let continue_rendering = self.update_world();
                if continue_rendering {
                    self.draw();
                } else {
                    self.game_over();
                    return false;
                }
            }
        }
        true
    }

    fn enough_time_passed(&mut self, timestamp :u32) -> bool {
        if timestamp > self.timestamp_last_frame {
            if (timestamp - self.timestamp_last_frame) as i32 > self.frame_time_threshold() {
                self.timestamp_last_frame = timestamp;
                return true;
            }
        }
        false
    }

    fn frame_time_threshold(&self) -> i32 {
        let speed_increase = if self.touch_mode {SPEED_INCREASE_MS_MODE_TOUCH} else {SPEED_INCREASE_MS_MODE_KEYBOARD};
        let speed_to_use = if self.pressed && self.game_mode == GameMode::LONG {
            6
        } else {
            self.speed
        };
        FRAME_RATE_SPEED_1 - speed_to_use * speed_increase
    }

    fn process_input(&mut self) {
        // don't allow opposite direction
        match &self.input {
            'a' => {
                if self.direction != Direction::RIGHT {
                    self.direction = Direction::LEFT
                }
            },
            's' => {
                if self.direction != Direction::UP {
                    self.direction = Direction::DOWN
                }
            },
            'd' => {
                if self.direction != Direction::LEFT {
                    self.direction = Direction::RIGHT
                }
            },
            'w' => {
                if self.direction != Direction::DOWN {
                    self.direction = Direction::UP
                }
            },
            ' ' => {
                log!("toggling pause");
                self.pause = !self.pause;
                self.input = DEFAULT_INPUT;
            }
            _ => (),
        }
    }

    fn update_world(&mut self) -> bool {
        self.move_snake();
        self.handle_food_collision();
        if self.check_border_collision() {
            return false;
        }
        if self.check_snake_collision() {
            return false;
        }
        true
    }

    fn move_snake(&mut self) {
        let mut i = 0;
        let mut prev = self.snake[0];
        for point in &mut self.snake {
            if i == 0 {
                match &self.direction {
                    Direction::UP => {
                        point.y -= 1;
                    },
                    Direction::DOWN => {
                        point.y += 1;
                    },
                    Direction::LEFT => {
                        point.x -= 1;
                    },
                    Direction::RIGHT => {
                        point.x += 1;
                    },
                }
            } else {
                let tmp = prev;
                prev = *point;
                *point = tmp;
            }
            i = i + 1;
        }
    }

    fn handle_food_collision(&mut self) {
        let point = &self.snake[0];
        let touch_mode_collision = if self.touch_mode {
            let relevant_points = vec![
                Point{x: self.snake[0].x - 1, y: self.snake[0].y - 1},
                Point{x: self.snake[0].x - 1, y: self.snake[0].y},
                Point{x: self.snake[0].x    , y: self.snake[0].y - 1},
                Point{x: self.snake[0].x + 1, y: self.snake[0].y},
                Point{x: self.snake[0].x    , y: self.snake[0].y + 1},
                Point{x: self.snake[0].x + 1, y: self.snake[0].y + 1},
            ];
            relevant_points.contains(&self.food)
        } else {
            false
        };
        if point == &self.food || touch_mode_collision {
            let last_point = self.snake[self.snake.len() - 1];
            self.snake.push(last_point);
            self.score = self.score + 1;
            self.inc_colour_index();
            if self.game_mode == GameMode::FAST {
                if self.score % SPEED_INCREASE_AT_SCORE == 0 {
                    self.speed = self.speed + 1;
                    log!("frame_time_threshold: {}", self.frame_time_threshold());
                    if self.speed >= SPEED_TO_SET_BG_COL {
                        let bg_col_idx = (self.speed - SPEED_TO_SET_BG_COL) as usize % BACKGROUND_COLOURS.len();
                        set_background_colour(BACKGROUND_COLOURS[bg_col_idx]);
                    }
                }
            }
            update_text_display(self.score, self.speed as u32);
            self.place_food();
        }
    }

    fn place_food(&mut self) {
        let mut point = Point{x: 0, y: 0};

        let mut food_x_min :u32 = 0;
        let mut food_y_min :u32 = 0;
        let mut food_x_max :i32 = self.calc_point_compontent(self.width);
        let mut food_y_max :i32 = self.calc_point_compontent(self.height);

        if self.touch_mode {
            food_x_min = TOUCH_MODE_FOOD_BORDER_OFFSET;
            food_y_min = TOUCH_MODE_FOOD_BORDER_OFFSET;
            food_x_max = food_x_max - TOUCH_MODE_FOOD_BORDER_OFFSET as i32;
            food_y_max = food_y_max - TOUCH_MODE_FOOD_BORDER_OFFSET as i32;
        }

        loop {
            point.x = random(food_x_min, food_x_max);
            point.y = random(food_y_min, food_y_max);

            if !self.snake.contains(&point) {
                break;
            }
        }
        log!("placing food at: {},{}", point.x, point.y);
        self.food = point;
    }

    fn inc_colour_index(&mut self) {
        if self.colour_index == COLOURS.len() -1 {
            self.colour_index = 0;
        } else {
            self.colour_index = self.colour_index + 1;
        }
    }

    fn check_border_collision(&self) -> bool {
        let point = &self.snake[0];
        match &self.direction {
            Direction::UP => {
                if point.y < 0 {
                    return true;
                }
            },
            Direction::DOWN => {
                if point.y >= self.calc_point_compontent(self.height) {
                    return true;
                }
            },
            Direction::LEFT => {
                if point.x < 0 {
                    return true;
                }
            },
            Direction::RIGHT => {
                if point.x > self.calc_point_compontent(self.width) {
                    return true;
                }
            },
        }
        return false;
    }

    fn check_snake_collision(&self) -> bool {
        let first_point = self.snake[0];
        let rest_of_snake = &self.snake[1..self.snake.len()];
        return rest_of_snake.contains(&first_point);
    }

    fn game_over(&mut self) {
        log!("game over");
        self.over = true;
        self.draw_game_over();
        let mode = if self.touch_mode {"Touch"} else {"Keyboard"};
        let latest_timestamp = highscore::add_score(&self.name, self.score, mode);
        highscore::print_highscores(latest_timestamp);
    }

    fn calc_center(&self) -> Point {
        Point {
            x: (self.width / self.block_size / 2) as i32,
            y: (self.height / self.block_size / 2) as i32,
        }
    }
    fn calc_coord(&self, point: &Point) -> Point {
        Point {
            x: point.x * self.block_size as i32,
            y: point.y * self.block_size as i32,
        }
    }
    fn calc_point_compontent(&self, component: u32) -> i32 {
        (component / self.block_size).try_into().unwrap()
    }

    fn draw(&self) {
        if self.context.is_some() {
            let context: &web_sys::CanvasRenderingContext2d = self.context.as_ref().unwrap();
            self.draw_clear(context);
            if self.draw_grid {
                self.draw_coord_sys(context);
            }
            self.draw_snake(context);
            self.draw_food(context);
        }
    }
    fn draw_clear(&self, context: &web_sys::CanvasRenderingContext2d) {
        context.set_fill_style(&JsValue::from_str("#FFF"));
        context.fill_rect(0.0, 0.0, self.width.into(), self.height.into());
    }
    fn draw_coord_sys(&self, context: &web_sys::CanvasRenderingContext2d) {
        let mut i = 0;
        context.set_stroke_style(&JsValue::from_str("#999"));
        context.set_line_width(1.0);
        while i < self.width {
            context.begin_path();
            context.move_to(i.into(), 0.into());
            context.line_to(i.into(), self.height.into());
            context.close_path();
            context.stroke();
            i += self.block_size;
        }
        i = 0;
        while i < self.height {
            context.begin_path();
            context.move_to(0.into(), i.into());
            context.line_to(self.width.into(), i.into());
            context.close_path();
            context.stroke();
            i += self.block_size;
        }
    }
    fn draw_snake(&self, context: &web_sys::CanvasRenderingContext2d) {
        let mut i :usize = 0;
        for point in &self.snake {
            let colour = if i < INITIAL_SNAKE_LEN {
                "#000"
            } else {
                COLOURS[(i - INITIAL_SNAKE_LEN) % COLOURS.len()]
            };
            self.draw_point(point, colour, context);
            i = i + 1;
        }
    }
    fn draw_point(&self, point: &Point, colour: &str, context: &web_sys::CanvasRenderingContext2d) {
        let coord = self.calc_coord(&point);
        context.set_fill_style(&JsValue::from_str(colour));
        context.fill_rect(coord.x.into(), coord.y.into(), self.block_size.into(), self.block_size.into());
    }
    fn draw_food(&self, context: &web_sys::CanvasRenderingContext2d) {
        context.set_stroke_style(&JsValue::from_str(COLOURS[self.colour_index]));
        context.set_line_width(3.0);
        let block_size_half:i32 = (self.block_size / 2) as i32;
        let mut coord = self.calc_coord(&self.food);
        coord.x = coord.x + block_size_half;
        coord.y = coord.y + block_size_half;
        context.begin_path();
        context
            .arc(coord.x.into(), coord.y.into(), block_size_half.into(), 0.0, std::f64::consts::PI * 2.0)
            .unwrap();
        context.stroke();

    }
    fn draw_pause(&self) {
        let context: &web_sys::CanvasRenderingContext2d = self.context.as_ref().unwrap();
        context.set_font("bold 30px serif");
        context.set_text_align("center");
        context.set_fill_style(&JsValue::from_str("#00F"));
        let result = context.fill_text("PAUSE", (self.width / 2).into(), (self.height / 2).into());
        crate::utils::handle_js_error(result);
    }

    fn draw_game_over(&self) {
        let context: &web_sys::CanvasRenderingContext2d = self.context.as_ref().unwrap();
        context.set_font("bold 30px serif");
        context.set_text_align("center");
        context.set_fill_style(&JsValue::from_str("#F00"));
        let result = context.fill_text("GAME OVER", (self.width / 2).into(), (self.height / 2).into());
        crate::utils::handle_js_error(result);
    }
}
