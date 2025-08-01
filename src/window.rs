use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

extern crate sdl2;
use sdl2::image::ImageRWops;
use sdl2::image::LoadSurface;
use sdl2::image::LoadTexture;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render;
use sdl2::rwops;
use sdl2::surface;
use sdl2::Sdl;

use crate::event::{self, Event};
use crate::shape;
use crate::util;

/**
 * A Window can display graphics and handle events.
 *
 * A Window has a draw color at all times, and that color is applied to every operation. If you set
 * the color to `(255, 0, 0)`, all drawn graphics and images will have a red tint.
 *
 * Creating multiple Windows is untested and will probably crash!
 *
 */
pub struct Window {
    // sdl graphics
    event_pump: sdl2::EventPump,
    timer_subsystem: sdl2::TimerSubsystem,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    foreground_color: pixels::Color,
    font: Option<Font>,

    // events and event logic
    running: bool,
    event_queue: std::vec::Vec<Event>,

    // timing
    target_ticks_per_frame: u32,
    ticks_at_previous_frame: u32,
}

/// Top-level Running / Creation Methods
/// ====================================
impl Window {
    /// Intialize a new running window. `name` is used as a caption.
    pub fn new(name: &str, width: u16, height: u16) -> Self {
        // SDL2 Initialization calls. This section here is the reason we can't easily create
        // multiple Windows. There would have to be some kind of global variable that tracked
        // whether SDL2 had already been init'd.
        //
        // Note that initialization is not the only problem. SDL2 is usually safe to init
        // multiple times, but it's not safe to de-init SDL2 and then continue using it. We'd
        // either have to have an explicit Deinitialize() global function or keep a global count
        // of windows that exist.
        //
        // Both solutions are ugly and error-prone, and would probably break thread safety. Going
        // to assume that there will only be one Window per program.
        //
        // TODO: solve this problem
        //
        let sdl_context = sdl2::init().unwrap();
        let timer_subsystem = sdl_context.timer().unwrap();
        sdl2::image::init(sdl2::image::InitFlag::all()).unwrap();

        let video_subsystem = sdl_context.video().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        let sdl_window_builder = video_subsystem.window(name, width as u32, height as u32);
        let sdl_window = sdl_window_builder.build().unwrap();
        let mut canvas = sdl_window.into_canvas().build().unwrap();

        // for transparency
        canvas.set_blend_mode(render::BlendMode::Blend);

        let mut window = Window {
            timer_subsystem,
            event_pump,
            canvas,
            running: true,
            event_queue: vec![],
            foreground_color: pixels::Color::RGBA(0, 0, 0, 255),
            target_ticks_per_frame: (1000.0 / 60.0) as u32,
            ticks_at_previous_frame: 0,
            font: None,
        };

        // clear first, then load the default font
        window.clear();
        window.canvas.present();
        window.set_color(255, 255, 255, 255);

        // load the default font
        let font = window
            .load_font(DEFAULT_FONT_BYTES, DEFAULT_FONT_STR.to_string())
            .unwrap();
        window.font = Some(font);

        window
    }

    /// Redrawing and update the display, while maintaining a consistent framerate and updating the
    /// event queue. You should draw your objects immediately before you call this function.
    ///
    /// NOTE: This function returns false if the program should terminate. This allows for nice
    /// constructs like `while app.next_frame() { ... }`
    pub fn next_frame(&mut self) -> bool {
        if !self.running {
            return false;
        }

        self.canvas.present();

        let mut current_ticks = self.timer_subsystem.ticks();
        while current_ticks - self.ticks_at_previous_frame < self.target_ticks_per_frame {
            self.timer_subsystem.delay(3);
            current_ticks = self.timer_subsystem.ticks();
        }
        self.ticks_at_previous_frame = current_ticks;

        // Handle events
        loop {
            let sdl_event = self.event_pump.poll_event();
            match sdl_event {
                None => break,
                Some(sdl_event) => match Event::from_sdl2_event(sdl_event) {
                    Some(Event::Quit) => self.quit(),

                    // any other unrecognized event
                    Some(e) => self.event_queue.push(e),
                    None => (),
                },
            };
        }

        true
    }

    /// Return true when there is an event waiting in the queue for processing.
    pub fn has_event(&self) -> bool {
        !self.event_queue.is_empty()
    }

    /// Get the next event from the queue. NOTE: If the event queue on the Window is empty, this
    /// function will panic. Call `has_event()` to find out if there is an event ready for
    /// processing.
    ///
    /// Note that events are handled in a first-in-first-out order. If a user presses three keys 1,
    /// 2, 3 during a frame, then the next three calls to next_event will return 1, 2, 3 in the
    /// same order.
    pub fn next_event(&mut self) -> Event {
        self.event_queue.remove(0)
    }

    /// Return true if the button is currently pressed. NOTE: This function is probably not
    /// performant.
    pub fn is_key_down(&self, key: event::Key) -> bool {
        self.event_pump.keyboard_state().is_scancode_pressed(key)
    }

    /// Return true if the specified button is down. NOTE: Unknown mouse buttons are NOT handled
    /// and will always return `false`.
    pub fn is_mouse_button_down(&self, button: event::MouseButton) -> bool {
        let mouse_state = self.event_pump.mouse_state();
        mouse_state.is_mouse_button_pressed(button)
    }

    /// Return the current position of the mouse, relative to the top-left corner of the Window.
    pub fn mouse_position(&self) -> (i32, i32) {
        let mouse_state = self.event_pump.mouse_state();
        (mouse_state.x(), mouse_state.y())
    }

    /// Use this Font for future calls to `print()`.
    pub fn set_font(&mut self, font: Font) {
        self.font = Some(font)
    }

    /// This does not cause the program to exit immediately. It just means that next_frame
    /// will return false on the next call.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Get the width and height of the screen
    fn get_max_dim() -> Result<Rect, Box<dyn Error>> {
        let sdl_context: Sdl = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        Ok(video_subsystem.display_bounds(0)?)
    }
    pub fn get_max_height() -> Result<u32, Box<dyn Error>> {
        Ok(Self::get_max_dim()?.height())
    }
    pub fn get_max_width() -> Result<u32, Box<dyn Error>> {
        Ok(Self::get_max_dim()?.width())
    }
}

/// Drawing Methods
/// ===============
impl Window {
    /// Windows have a color set on them at all times. This color is applied to every draw
    /// operation. To "unset" the color, call set_color with (255,255,255,255)
    pub fn set_color(&mut self, red: u8, green: u8, blue: u8, alpha: u8) {
        self.foreground_color = pixels::Color::RGBA(red, green, blue, alpha);
    }

    /// Set up the color according to the internal state of the Window.
    fn prepare_to_draw(&mut self) {
        self.canvas.set_draw_color(self.foreground_color);
    }

    // These functions are just aliases onto self.canvas.as you can see.
    pub fn draw_rect(&mut self, rect: shape::Rect) {
        self.prepare_to_draw();
        self.canvas.draw_rect(rect).unwrap();
    }
    pub fn fill_rect(&mut self, rect: shape::Rect) {
        self.prepare_to_draw();
        self.canvas.fill_rect(rect).unwrap();
    }
    pub fn draw_point(&mut self, point: shape::Point) {
        self.prepare_to_draw();
        self.canvas.draw_point(point).unwrap();
    }
    pub fn draw_polygon(&mut self, polygon: shape::Polygon) {
        self.prepare_to_draw();
        self.canvas.draw_points(&polygon[..]).unwrap();
    }

    /// Display the image with its top-left corner at (x, y)
    pub fn draw_image(&mut self, image: &mut Image, x: i32, y: i32) {
        // first, configure the texture for drawing according to the current foreground_color
        util::set_texture_color(&self.foreground_color, &mut image.texture);

        // copy the texture onto the drawer()
        self.canvas
            .copy(
                &(image.texture),
                Some(shape::Rect::new(
                    x,
                    y,
                    image.get_width(),
                    image.get_height(),
                )),
                None,
            )
            .unwrap();
    }

    /// Write the text to the screen at (x, y) using the currently set font on the Window. Return a
    /// Rectangle describing the area of the screen that was modified.
    // TODO: Implement print_rect that wraps text to fit inside of a Rectangle.
    pub fn print(&mut self, text: &str, x: i32, y: i32) -> shape::Rect {
        self.prepare_to_draw();
        let font = match self.font {
            Some(ref mut r) => r,

            // FIXME: shouldn't be possible to have no font, and the `font` field on Window should
            // be updated to reflect this.
            None => panic!("no font set on window"),
        };
        util::set_texture_color(&self.foreground_color, &mut font.texture);

        let mut current_x = x;

        for ch in text.chars() {
            let font_rect = match font.get_rect(ch) {
                None => {
                    // Our Font cannot represent the current character. Leave a little space.
                    current_x += 5;
                    continue;
                }
                Some(r) => r,
            };

            let rect = shape::Rect::new(current_x, y, font_rect.width(), font_rect.height());
            self.canvas
                .copy(&(font.texture), Some(*font_rect), rect)
                .unwrap();

            current_x += font_rect.width() as i32;
        }

        shape::Rect::new(x, y, (current_x - x) as u32, font.get_height() as u32)
    }

    /// Clear the screen to black. Does not affect the current rendering color.
    pub fn clear(&mut self) {
        self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    /// Clear the screen to the color you specify.
    pub fn clear_to_color(&mut self, r: u8, g: u8, b: u8) {
        self.canvas.set_draw_color(pixels::Color::RGB(r, g, b));
        self.canvas.clear();
    }
}

/**
 * Image represents a texture that can be drawn on the screen.
 *
 * Images are immutable, in the sense that they have no methods to modify their contents.
 */
pub struct Image {
    texture: render::Texture,
    width: u32,
    height: u32,
}

impl Image {
    pub fn get_width(&self) -> u32 {
        self.width
    }
    pub fn get_height(&self) -> u32 {
        self.height
    }
}

/**
 * Font is a way to render text, loaded from a specially formatted image.
 *
 * Note that Font is not loaded from a TrueType file, but instead, from a specially formatted
 * image. Loading from an image is a little faster and a little simpler and a little more portable,
 * but has a couple disadvantages. For one, the font size is fixed by the file. To have two
 * different font sizes, you have to create two different Fonts from two different files. Another
 * disadvantage is that these special images are less widely available.
 *
 * This link describes how ImageFonts work: https://love2d.org/wiki/Tutorial:Fonts_and_Text
 */
pub struct Font {
    texture: render::Texture,
    chars: HashMap<char, shape::Rect>,
    height: u32,
}

impl Font {
    /// Determine whether "ch" exists in this Font.
    pub fn is_printable(&self, ch: char) -> bool {
        self.chars.contains_key(&ch)
    }

    /// Return the number of printable characters that the Font contains.
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// Returns `true` if the `chars` contains no elements.
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// Return the height of the Font. This is constant for every possible character, while the
    /// individual character widths vary. Note that certain characters (such a single quote `'`)
    /// might not actually take up all of `height`. However, no character may exceed this limit.
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Return the portion of the Font's texture that is used to draw the `char` you provide. If
    /// the character can't be drawn by this Font, return None.
    fn get_rect(&self, ch: char) -> Option<&shape::Rect> {
        self.chars.get(&ch)
    }
}

/// This is the default font.
const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("default_font.png");
const DEFAULT_FONT_STR: &str =
    " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.,!?-+/():;%&`'*#=[]\"";

/// Resource Loading Methods
/// ========================
impl Window {
    /// Load the image at the path you specify.
    pub fn load_image_from_file(&self, filename: &Path) -> Result<Image, String> {
        let mut texture = self.canvas.texture_creator().load_texture(filename)?;
        texture.set_blend_mode(render::BlendMode::Blend);
        Ok(Image {
            width: texture.query().width,
            height: texture.query().height,
            texture,
        })
    }

    /// Load an image from a slice of bytes. This function is particularly powerful when
    /// used in conjunction with the `include_bytes` macro that embeds data in the compiled
    /// executable. In this way, you can pack all of your game data into your executable.
    pub fn load_image(&self, data: &[u8]) -> Result<Image, String> {
        let rwops = rwops::RWops::from_bytes(data)?;
        let surf: surface::Surface = rwops.load()?;
        let mut texture = match self
            .canvas
            .texture_creator()
            .create_texture_from_surface(&surf)
        {
            Ok(t) => t,
            Err(e) => return Err(e.to_string()),
        };
        texture.set_blend_mode(render::BlendMode::Blend);
        Ok(Image {
            width: texture.query().width,
            height: texture.query().height,
            texture,
        })
    }

    // TODO: Split this out so it can be tested.

    /// Parse a font from the Surface, using the string as a guideline.
    fn parse_image_font(&self, surf: surface::Surface, string: String) -> Result<Font, String> {
        if util::string_has_duplicate_chars(string.clone()) {
            return Err("image font string has duplicate characters".to_string());
        }

        let surf = surf;
        let mut chars: HashMap<char, shape::Rect> = HashMap::new();

        // let surf_width = surf.width();
        let surf_height = surf.height();
        let mut current_rect: Option<shape::Rect> = None;

        surf.with_lock(|pixels| {
            // `pixels` is an array of [u8; width * height]
            let border_color = pixels[0];

            // Move through the surface and divide it into rectangles according to the color of the
            // topmost pixel.
            for (i, pixel) in pixels.iter().enumerate() {
                // for i in 0..(surf_width as usize) {
                if pixel == &border_color {
                    if let Some(mut rect) = current_rect {
                        let c = match string.chars().nth(chars.len()) {
                            Some(c) => c,
                            None => {
                                // Out of characters to add to the hashmap, so just return with
                                // what have parsed so far.
                                return;
                            }
                        };
                        rect = shape::Rect::new(
                            rect.x(),
                            rect.y(),
                            ((i as i32) - rect.x()) as u32,
                            rect.height(),
                        );
                        chars.insert(c, rect);
                        current_rect = None;
                    }
                } else if current_rect.is_none() {
                    current_rect = Some(shape::Rect::new(i as i32, 0, 1, surf_height));
                }
            }
        });

        let mut texture = match self
            .canvas
            .texture_creator()
            .create_texture_from_surface(&surf)
        {
            Ok(t) => t,
            Err(e) => return Err(e.to_string()),
        };
        texture.set_blend_mode(render::BlendMode::Blend);
        Ok(Font {
            height: texture.query().height,
            texture,
            chars,
        })
    }

    /// Load a Font from the hard drive. See the documentation on `Font` for details.
    pub fn load_font_from_file(&self, filename: &Path, string: String) -> Result<Font, String> {
        let surf: surface::Surface = LoadSurface::from_file(filename)?;
        self.parse_image_font(surf, string)
    }

    /// Load a Font from a slice of bytes. See the documentation on `Font` for details. This
    /// function is particularly powerful when used in conjunction with the `include_bytes` macro
    /// that embeds data in the compiled executable.
    pub fn load_font(&self, data: &[u8], string: String) -> Result<Font, String> {
        let rwops = rwops::RWops::from_bytes(data)?;
        let surf: surface::Surface = rwops.load()?;
        self.parse_image_font(surf, string)
    }
}
