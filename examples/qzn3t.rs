/*!
 * A copy of `squares`
 *
 * This example uses the draw methods on simple::Window to display
 * some rectangles bouncing around on screen. You can click anywhere
 * to add a new rectangle.
 *
 * Added features:
 */

extern crate rand;
use std::f32;

use rand::random;

extern crate simple;
use simple::event::MouseEventType;
use simple::{Event, Rect, Window};
use std::time::Duration;
use std::time::Instant;
/// Return an f32 in the interval [0, upper_bound]
/// Used to generate random positions for Square.
fn rand_up_to(upper_bound: f32) -> f32 {
    random::<f32>().abs() * upper_bound
}

/// Square is our game object. It has a position, movement vector, and color.
#[derive(Debug, Copy, Clone)]
struct Square {
    x: f32,
    y: f32,
    speed_x: f32,
    speed_y: f32,
    color: (u8, u8, u8, u8),
    screen_width: u32,
    screen_height: u32,
    width: u32,
    height: u32,
}

impl Square {
    /// Generate a Square with random speed and color starting at the point you specify
    fn new_at_position(x: f32, y: f32, screen_width: u32, screen_height: u32) -> Self {
        // generate a random angle and then use that angle to calculate the initial speed_x and
        // speed_y. We do this because it simplifies bouncing logic later in the update function.
        // The multiplication here is because random::<f32> appears to generate a value between 0
        // and 1, so we have to expand that range to [0, 2*PI] to get a full distribution of
        // possible angles.
        let angle: f32 = rand_up_to(f32::consts::PI * 2.0);

        Square {
            x,
            y,
            speed_x: angle.sin() * 8.0,
            speed_y: angle.cos() * 8.0,
            color: (random(), random(), random(), 255), // color is totally random
            screen_width,
            screen_height,
            width: 64,
            height: 64,
        }
    }

    /// Generate a totally random new Square
    fn new(screen_width: u32, screen_height: u32) -> Self {
        Square::new_at_position(
            rand_up_to(screen_width as f32),
            rand_up_to(screen_height as f32),
            screen_width,
            screen_height,
        )
    }

    /// Move the Square the distance it needs to travel for one frame.
    /// The speed slows down, and gravity effects it too
    fn update(&mut self) {
        self.x += self.speed_x;
        self.y += self.speed_y;
        if self.x < 0.0 || self.x > self.screen_width as f32 {
            self.speed_x *= -1f32;
        }
        // if self.y < 0.0 || self.y > self.screen_height as f32 {
        //     self.speed_y *= -1f32;
        // }

        // Slow down
        self.speed_x *= 0.99;
        self.speed_y *= 0.99;

        // Acceleration due to gravity.
        self.speed_y -= 1.0;
        if self.y <= self.height as f32 / 2.0 {
            // eprintln!("Stopping: {self:?}");
            // // Bounce

            // Stop
            self.speed_y = 0.0;
            self.speed_x = 0.0;
            self.y = self.height as f32 / 2.0;
            self.speed_y = rand_up_to(40.0);
            self.speed_x = rand_up_to(20.0) - 10.0;
        }

        if self.speed_y <= f32::EPSILON {}
    }

    /// Blit a square representing this object onto the Window.
    fn draw(&self, app: &mut Window) {
        app.set_color(self.color.0, self.color.1, self.color.2, self.color.3);
        // eprintln!("Draw: {self:?}");

        let sw = self.screen_width;
        let sh = self.screen_height;
        let y_p = self.y / sh as f32;
        let x_p = self.x / sw as f32;
        if !(0.0..1.0).contains(&x_p) {
            eprintln!("x_p: {x_p:0.3}");
        }
        if !(0.0..1.0).contains(&y_p) {
            eprintln!("y_p: {y_p:0.3}");
        }

        let x = sw as f32 * y_p;
        let y = sh as f32 - sh as f32 * x_p;

        if !(0.0..sh as f32).contains(&x) {
            eprintln!("Width: {sw} x: {x}");
        }
        if !(0.0..sh as f32).contains(&y) {
            eprintln!("Height: {sh} y: {y}");
        }

        // let x = sw as f32 - self.x;
        // let y = sh as f32 - self.y;
        let draw_rect = Rect::new(
            x as i32 - self.width as i32 / 2,
            y as i32 - self.height as i32 / 2,
            self.width,
            self.height,
        );

        let _ = app.fill_rect(draw_rect);
    }
}

fn main() {
    // Create an application

    // Get screen dimensions
    let (max_w, max_h) = Window::get_max_wh().unwrap();
    let (max_w, max_h) = (max_w / 2, max_h / 2);
    let mut app = Window::new("Squares", max_w as u16, max_h as u16);
    let (w, h) = app.drawable_size();
    // Create some objects to live in the application
    // let mut squares = vec![Square::new(w, h), Square::new(w, h), Square::new(w, h)];
    let mut squares = vec![Square::new(w, h)];

    // Run the game loop
    let mut now = Instant::now();
    while app.next_frame() {
        if now.elapsed() < Duration::from_millis(100) {
            continue;
        }
        now = Instant::now();
        // event handling
        while app.has_event() {
            if let Event::Mouse {
                event_type: MouseEventType::Down,
                mouse_x,
                mouse_y,
                ..
            } = app.next_event()
            {
                // If the user clicks, we add a new Square at the position of the mouse event.
                squares.push(Square::new_at_position(
                    mouse_x as f32,
                    mouse_y as f32,
                    w,
                    h,
                ));
            }
        }

        app.clear();

        // update and draw
        for square in squares.iter_mut() {
            square.update();
            square.draw(&mut app);
        }
    }
}
