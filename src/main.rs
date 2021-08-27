use pixels::{Error, Pixels, SurfaceTexture};
use std::iter::{once, repeat};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Battleship!")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| println!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            world.update(&input);
            window.request_redraw();
        }
    });
}

const WIDTH: u32 = 720;
const HEIGHT: u32 = 600;

/// Representation of the application state
struct World {
    this_player: Player,
    other_player: Player,
}

// 0-indexed grid positions
struct Cell {
    x: u8,
    y: u8,
}

struct Ship {
    len: u8,
    cells: Vec<Cell>,
}

impl Ship {
    fn starting_five() -> [Self; 5] {
        [
            Ship {
                len: 2,
                cells: vec![Cell { x: 0, y: 0 }, Cell { x: 1, y: 0 }],
            },
            Ship {
                len: 3,
                cells: Vec::new(),
            },
            Ship {
                len: 3,
                cells: Vec::new(),
            },
            Ship {
                len: 4,
                cells: Vec::new(),
            },
            Ship {
                len: 5,
                cells: Vec::new(),
            },
        ]
    }
}

struct Player {
    ships: [Ship; 5],
    shots_taken: Vec<Cell>,
}

impl Player {
    fn new() -> Self {
        Player {
            ships: Ship::starting_five(),
            shots_taken: Vec::new(),
        }
    }
}

// type alias for colors
type Color = [u8; 4];

const WHITE: Color = [0xff, 0xff, 0xff, 0xff]; // FFFFFF
const BLACK: Color = [0x00, 0x00, 0x00, 0xff];
const DARK_GREEN: Color = [0x20, 0x2a, 0x25, 0xff]; // 202A25
const GRAY: Color = [0xEB, 0xE9, 0xE9, 0xff]; //EBE9E9
const GREEN: Color = [0x00, 0xA8, 0x78, 0xff]; // 00A878
const YELLOW: Color = [0xf8, 0xf3, 0x2b, 0xff]; // F8F32B
const BLUE: Color = [0x6c, 0xcf, 0xf6, 0xff]; // 6CCFF6
const FLAME: Color = [0xcf, 0x5c, 0x36, 0xff]; // CF5C36

const GRID_LINES: Color = GRAY;
const GRID_EMPTY: Color = BLUE;
const BACKGROUND: Color = DARK_GREEN;

impl World {
    /// Draw the `World` state to the frame buffer.
    fn draw(&self, frame: &mut [u8]) {
        let top_margin = 190;
        //
        // top frame pixels
        //
        for i in 0..top_margin {
            let i = i as usize;
            let w = WIDTH as usize;
            let pixels = BACKGROUND
                .iter()
                .cycle()
                .cloned()
                .take(w * 4)
                .collect::<Vec<_>>();
            frame[i * w * 4..(i + 1) * w * 4].copy_from_slice(&pixels);
        }

        let grid_width = 301;
        let grid_margin = 40;
        let cell_width = 30;

        //
        // draw two grids
        //
        for line in 0..grid_width {
            let i = (line + top_margin) as usize;
            let w = WIDTH as usize;
            let grid_pixels: Vec<u8> = if line % cell_width == 0 {
                repeat(GRID_LINES).take(grid_width).flatten().collect()
            } else {
                // empty
                once(GRID_LINES)
                    .chain(repeat(GRID_EMPTY).take(29))
                    .cycle()
                    .take(grid_width)
                    .flatten()
                    .collect()
            };

            let margin: Vec<u8> = repeat(BACKGROUND).take(grid_margin).flatten().collect();
            let pixels: Vec<u8> = margin
                .iter()
                .chain(grid_pixels.iter())
                .chain(margin.iter())
                .chain(grid_pixels.iter())
                .chain(margin.iter())
                .cloned()
                .take(w * 4)
                .collect();

            frame[i * w * 4..(i + 1) * w * 4].copy_from_slice(&pixels);
        }
        // 150px of empty (dark green)
        for i in top_margin + grid_width..HEIGHT as usize {
            let i = i as usize;
            let w = WIDTH as usize;
            let pixels = BACKGROUND
                .iter()
                .cycle()
                .cloned()
                .take(w * 4)
                .collect::<Vec<_>>();
            frame[i * w * 4..(i + 1) * w * 4].copy_from_slice(&pixels);
        }

        for ship in self.this_player.ships.iter() {
            for cell in &ship.cells {
                World::fill_cell(cell, frame, WHITE, true);
            }
        }

        for ship in self.other_player.ships.iter() {
            for cell in &ship.cells {
                World::fill_cell(cell, frame, GRAY, false);
            }
        }
    }

    fn fill_cell(cell: &Cell, frame: &mut [u8], color: Color, this_player: bool) {
        let grid_width = 301;
        let grid_margin = 40;
        let cell_width = 30;
        let cell_margin = 4;
        let top_margin = 190;

        // cell width and height
        let filled_len = cell_width - 2 * cell_margin;
        // one line _across_ within a filled cell
        let line: Vec<u8> = repeat(color).take(filled_len).flatten().collect();

        // whose cell is this?
        let grid_offset = if this_player {
            // left: just need grid_margin
            grid_margin
        } else {
            grid_width + 2 * grid_margin
        };

        for i in 0..filled_len {
            let y_offset =
                (top_margin + i + cell_width * cell.y as usize + cell_margin) * 4 * WIDTH as usize;
            let x_offset = (grid_offset + cell_width * cell.x as usize + cell_margin) * 4;
            let cell_start = y_offset + x_offset;
            frame[cell_start..cell_start + filled_len * 4].copy_from_slice(&line);
        }
    }

    /// Create a new `World` instance with empty values
    fn new() -> Self {
        World {
            this_player: Player::new(),
            other_player: Player::new(),
        }
    }

    /// Update the `World` internal state
    fn update(&mut self, input: &WinitInputHelper) {
        if input.key_pressed(VirtualKeyCode::Down) {}
        if input.key_pressed(VirtualKeyCode::Up) {}
        if input.key_pressed(VirtualKeyCode::Right) {}
        if input.key_pressed(VirtualKeyCode::Left) {}
    }
}
