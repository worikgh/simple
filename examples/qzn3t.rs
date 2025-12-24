/*!
 * A copy of `squares`
 *
 * This example uses the draw methods on simple::Window to display
 * some rectangles bouncing around on screen. You can click anywhere
 * to add a new rectangle.
 *
 * Added features:
 * Rotate the screen 90 degrees counter clockwise
 * Gravity effects squares to illustrate where "down" is
 * Squares that hit "left side" (x == 0) wrap around, squares that hit
 * "right side" (x == w) bounce (revrse direction) illustrating where
 * left side is
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
    model_width: u32,
    model_height: u32,
    width: u32,
    height: u32,
    trend_left: bool,
}

impl Square {
    /// Generate a Square with random speed and color starting at the point you specify
    fn new_at_position(x: f32, y: f32, model_width: u32, model_height: u32) -> Self {
        //  FIXME!!  x, y, screen_width and screen_height should all be same type
        // generate a random angle and then use that angle to calculate the initial speed_x and
        // speed_y. We do this because it simplifies bouncing logic later in the update function.
        // The multiplication here is because random::<f32> appears to generate a value between 0
        // and 1, so we have to expand that range to [0, 2*PI] to get a full distribution of
        // possible angles.
        let angle: f32 = rand_up_to(f32::consts::PI * 2.0);
        let trend_left = random::<u8>().is_multiple_of(2);
        Square {
            x,
            y,
            speed_x: angle.sin() * 8.0,
            speed_y: angle.cos() * 8.0,
            color: (random(), random(), random(), 255), // color is totally random
            model_width,
            model_height,
            width: 64,
            height: 64,
            trend_left, //: random::<u8>().is_multiple_of(2),
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
        // let start = format!(
        //     "@({:0.0},{:0.0}) dX:{:0.1} dY:{:0.1}  LEFT:{}",
        //     self.x, self.y, self.speed_x, self.speed_y, self.trend_left
        // );
        self.x += self.speed_x;
        self.y += self.speed_y;
        if self.x < 0.0 {
            // Hit "left side". Wrap around
            self.x = self.model_width as f32;
            if self.trend_left {
                // Slow it down
                self.speed_x *= 0.5;
            }
        } else if self.x > self.model_width as f32 {
            // Hit "right side"  Make veloicity leftwards
            self.speed_x = -self.speed_x.abs();
            if self.speed_x.abs() < 2.0 && !self.trend_left {
                self.speed_x = -rand_up_to(self.model_width as f32);
            }
        }

        if self.y > self.model_height as f32 {
            self.speed_y = -self.speed_y.abs();
            if self.speed_y.abs() < 2.0 {
                // fire from a cannon
                self.speed_y = -rand_up_to(self.model_height as f32);
            }
        }

        if self.y <= self.height as f32 / 2.0 {
            // Hit roof.  Up is lower Y
            self.y = self.height as f32 / 2.0;
            // self.speed_y = rand_up_to(self.model_height as f32);
            self.speed_y = rand_up_to(40.0);
        }

        // Slow down
        self.speed_x *= 0.99;
        self.speed_y *= 0.99;

        // Acceleration due to gravity.  Down is to higher y
        self.speed_y += 1.0;

        if self.speed_y.abs() <= f32::EPSILON {
            eprintln!("Speed_y is zero");
        }
        // Drift....
        self.speed_x += if self.trend_left { -0.1 } else { 0.1 };
        // let end = format!(
        //     "@({:0.0},{:0.0}) dX:{:0.1} dY:{:0.1}",
        //     self.x, self.y, self.speed_x, self.speed_y,
        // );
        // eprintln!(
        //     "{start} -> {end} {}x{}",
        //     self.model_width, self.model_height
        // );
    }

    /// Blit a square representing this object onto the Window.
    fn draw(&self, app: &mut Window) {
        {
            let r = Rect::new(self.x as i32, self.y as i32, self.width, self.height);
            fill_rect(app, r, self.color.into());
        }
        {
            let ind_rect = if self.trend_left {
                Rect::new(
                    self.x as i32,
                    self.y as i32 - self.height as i32 / 4,
                    self.width / 2,
                    self.height / 4,
                )
            } else {
                Rect::new(
                    (self.x as u32 + self.width / 2) as i32,
                    self.y as i32 - self.height as i32 / 2,
                    self.width / 2,
                    self.height / 4,
                )
            };

            fill_rect(app, ind_rect, [255, 255, 255, 127]);
        }
    }
}

/// This is where the transformation of where drawing happens on
/// screen is implemented.  Rotated anti-clockwise ninety degrees
fn fill_rect(app: &mut Window, rect: Rect, colour: [u8; 4]) {
    let ds = app.drawable_size();
    let view_w = ds.0;
    let view_h = ds.1;
    let (x, y) = Window::translate_model_90c(rect.x(), rect.y(), view_w, view_h);

    let draw_rect = Rect::new(x, y, rect.height(), rect.width());
    eprintln!("fill_rect: {rect:?} -> {draw_rect:?}  {view_w}x{view_h}");
    app.set_color(colour[0], colour[1], colour[2], colour[3]);
    app.fill_rect(draw_rect);
}
fn main() {
    // Create an application

    // Get screen dimensions
    let (max_w, max_h) = Window::get_max_wh().unwrap();
    let mut app = Window::new("Squares", max_w as u16, max_h as u16);
    let (view_w, view_h) = app.drawable_size();
    let (model_w, model_h) = (view_h, view_w);

    // Create some objects to live in the application
    let mut squares = vec![
        Square::new(model_w, model_h),
        Square::new(model_w, model_h),
        Square::new(model_w, model_h),
    ];

    // Run the main GUI loop
    let mut now = Instant::now();
    eprintln!("Screen: {view_w}x{view_h}");
    while app.next_frame() {
        if now.elapsed() < Duration::from_millis(100) {
            continue;
        }
        now = Instant::now();
        // event handling
        while app.has_event() {
            if let Event::Mouse {
                event_type: MouseEventType::Down,
                mouse_x: view_x,
                mouse_y: view_y,
                ..
            } = app.next_event()
            {
                // If the user clicks, we add a new Square at the
                // position of the mouse event.

                // Rotate mouse position from view to model
                let (model_x, model_y) = Window::translate_view_90c(view_x, view_y, view_w, view_h);
                squares.push(Square::new_at_position(
                    model_x as f32,
                    model_y as f32,
                    model_w,
                    model_h,
                ));
            }
        }

        app.clear_to_color(255, 255, 255);

        // update and draw
        for square in squares.iter_mut() {
            square.update();
            square.draw(&mut app);
        }

        // Draw brown rectangle on bottom of screen (max Y)
        {
            let bottom_rect = Rect::new(0, model_h as i32 - 10, model_w, 10);
            // eprintln!("bottom_rect: {bottom_rect:?} w:{view_w} h:{view_h}");
            fill_rect(&mut app, bottom_rect, [139, 69, 19, 127]);
        }
        // Draw cross
        {
            // vertical
            let rect = Rect::new(model_w as i32 / 2 - 10, 0, 20, model_h);
            eprintln!("Vertical:   {rect:?}  {model_w}x{model_h}");
            fill_rect(&mut app, rect, [255, 0, 0, 127]);
        }
        {
            // horizontal
            let rect = Rect::new(0, model_h as i32 / 2 - 10, model_w, 20);
            eprintln!("Horizontal: {rect:?}  {model_w}x{model_h}");
            fill_rect(&mut app, rect, [255, 0, 0, 255]);
        }
    }
}
