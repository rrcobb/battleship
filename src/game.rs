use rand::prelude::Distribution;
use rand::{distributions::Standard, rngs::ThreadRng, thread_rng, Rng};
use rusttype::{point, Font, Scale};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;
use std::iter::{once, repeat};
use std::convert::From;

use crate::connection::*;
use std::net::TcpStream;

/// Representation of the application state, plus some
pub struct World<'a> {
    font: Font<'a>,
    rng: ThreadRng,
    status: GameStatus,
    stream: Option<TcpStream>,
    this_player: Player,
    other_player: Player,
}

#[derive(Debug, Clone, PartialEq)]
enum GameResult {
    Victory,
    Defeat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameType {
    Ai,
    LocalNetwork,
}

#[derive(Debug, Clone, PartialEq)]
enum GameStatus {
    Playing,
    End(GameResult),
}

// 0-indexed grid positions
#[derive(Debug, Clone, PartialEq, Default)]
struct Cell {
    x: u8,
    y: u8,
}

#[derive(Debug, Clone, PartialEq)]
enum Move {
    Enter,
    Space,
    Right,
    Left,
    Up,
    Down,
}

impl From<Direction> for Move {
    fn from(item: Direction) -> Self {
        use Direction::*;
        match item {
            Up => Move::Up,
            Down => Move::Down,
            Right => Move::Right,
            Left => Move::Left,
        }
    }
}

impl Cell {
    fn shift(&mut self, direction: &Direction) -> bool {
        let (x_shift, y_shift) = direction.xy();
        let x = x_shift + self.x as i8;
        let y = y_shift + self.y as i8;
        let valid_shift = x >= 0 && x < CELL_COUNT as i8 && y >= 0 && y < CELL_COUNT as i8;
        if valid_shift {
            self.x = x as u8;
            self.y = y as u8;
        }
        valid_shift
    }

    fn random_seq(len: &u8, rng: &mut ThreadRng) -> Vec<Cell> {
        let cell: Cell = rng.gen();
        cell.extend(rng, len)
    }

    fn extend(&self, rng: &mut ThreadRng, len: &u8) -> Vec<Cell> {
        let mut cell = self.clone();
        let mut res = Vec::new();
        let mut valid = false;
        while !valid {
            valid = true;
            // for any valid cell, extending len in _some_ direction should be valid
            let direction = rng.gen();
            for _i in 0..*len {
                res.push(cell.clone());
                // ensure all cells are valid shifts
                valid = valid && cell.shift(&direction);
            }
        }
        res
    }
}

impl Distribution<Cell> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cell {
        Cell {
            x: rng.gen_range(0..10),
            y: rng.gen_range(0..10),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
enum ShipStatus {
    #[default]
    Hidden,
    Placing,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Ship {
    status: ShipStatus,
    len: u8,
    cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        use Direction::*;
        let index: u8 = rng.gen_range(0..4);
        match index {
            0 => Up,
            1 => Down,
            2 => Left,
            3 => Right,
            _ => unreachable!(),
        }
    }
}

impl Direction {
    fn xy(&self) -> (i8, i8) {
        use Direction::*;
        match self {
            Up => (0, -1),
            Down => (0, 1),
            Left => (-1, 0),
            Right => (1, 0),
        }
    }
}

impl Ship {
    fn random_five(rng: &mut ThreadRng) -> Vec<Self> {
        use ShipStatus::*;
        let lengths = [2, 3, 4, 4, 5];
        let mut ships = Vec::new();
        for len in lengths.iter() {
            let cells = Cell::random_seq(len, rng);
            let mut ship = Ship {
                status: Locked,
                len: *len,
                cells,
            };
            while Ship::any_overlap(&ship, &ships) {
                let cells = Cell::random_seq(len, rng);
                ship = Ship {
                    status: Locked,
                    len: *len,
                    cells,
                };
            }
            ships.push(ship);
        }
        ships
    }

    fn original_length_ships() -> Vec<Self> {
        use ShipStatus::*;
        vec![
            Ship {
                status: Placing,
                len: 2,
                cells: vec![Cell { x: 0, y: 0 }, Cell { x: 1, y: 0 }],
            },
            Ship {
                status: Hidden,
                len: 3,
                cells: vec![
                    Cell { x: 0, y: 0 },
                    Cell { x: 1, y: 0 },
                    Cell { x: 2, y: 0 },
                ],
            },
            Ship {
                status: Hidden,
                len: 3,
                cells: vec![
                    Cell { x: 0, y: 0 },
                    Cell { x: 1, y: 0 },
                    Cell { x: 2, y: 0 },
                    Cell { x: 3, y: 0 },
                ],
            },
            Ship {
                status: Hidden,
                len: 4,
                cells: vec![
                    Cell { x: 0, y: 0 },
                    Cell { x: 1, y: 0 },
                    Cell { x: 2, y: 0 },
                    Cell { x: 3, y: 0 },
                ],
            },
            Ship {
                status: Hidden,
                len: 5,
                cells: vec![
                    Cell { x: 0, y: 0 },
                    Cell { x: 1, y: 0 },
                    Cell { x: 2, y: 0 },
                    Cell { x: 3, y: 0 },
                    Cell { x: 4, y: 0 },
                ],
            },
        ]
    }

    fn shift(&mut self, direction: &Direction) {
        let (x, y) = direction.xy();
        // move each cell in the direction it should be moved
        let mut valid = true;
        let mut shifted: Vec<Cell> = self.cells.clone();

        for (i, cell) in self.cells.iter().enumerate() {
            let x = x + cell.x as i8;
            let y = y + cell.y as i8;
            // if any cells end up out of bounds (< 0 or > grid_width), cancel the whole move operation
            if x >= 0 && x < CELL_COUNT as i8 && y >= 0 && y < CELL_COUNT as i8 {
                shifted[i] = Cell {
                    x: x as u8,
                    y: y as u8,
                };
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            self.cells = shifted;
        }
    }

    fn rotate_right(&mut self) {
        // for the cell i
        // move x,y i times
        // direction-finding: difference between cells[0] and cells[1]
        let one = &self.cells[0];
        let two = &self.cells[1];
        let (x, y): (i8, i8) = match (one.x as i8 - two.x as i8, one.y as i8 - two.y as i8) {
            (-1, 0) => (-1, 1),
            (0, -1) => (-1, -1),
            (1, 0) => (1, -1),
            (0, 1) => (1, 1),
            _ => panic!("adjacent cells were not adjacent!"),
        };
        let mut valid = true;
        let mut shifted: Vec<Cell> = self.cells.clone();
        for (i, cell) in self.cells.iter().enumerate() {
            let n: i8 = i as i8;
            let x = n * x + cell.x as i8;
            let y = n * y + cell.y as i8;
            // if any cells end up out of bounds (< 0 or > grid_width), cancel the whole operation
            if x >= 0 && x < CELL_COUNT as i8 && y >= 0 && y < CELL_COUNT as i8 {
                shifted[i] = Cell {
                    x: x as u8,
                    y: y as u8,
                };
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            self.cells = shifted;
        }
    }

    fn placing(&mut self) {
        self.status = ShipStatus::Placing;
    }

    fn any_overlap(ship: &Ship, ships: &[Ship]) -> bool {
        ships.iter().any(|other_ship| {
            other_ship.status != ShipStatus::Hidden
                && other_ship != ship
                && other_ship
                    .cells
                    .iter()
                    .any(|cell| ship.cells.contains(cell))
        })
    }
}

#[derive(Debug, PartialEq)]
enum PlayerStatus {
    Placing,
    Aiming,
    Waiting,
}

struct Player {
    status: PlayerStatus,
    ships: Vec<Ship>,
    target: Cell,
    shots_taken: Vec<Cell>,
}

impl Player {
    fn new() -> Self {
        Player {
            status: PlayerStatus::Placing,
            ships: Ship::original_length_ships(),
            target: Cell { x: 4, y: 5 },
            shots_taken: Vec::new(),
        }
    }

    fn ship_to_place(&self) -> Option<&Ship> {
        self.ships
            .iter()
            .find(|ship| ship.status == ShipStatus::Placing)
    }

    fn ship_to_place_mut(&mut self) -> Option<&mut Ship> {
        self.ships
            .iter_mut()
            .find(|ship| ship.status == ShipStatus::Placing)
    }

    fn fire(&mut self) -> bool {
        let target = &self.target;
        let overlaps_shot = self.shots_taken.iter().any(|shot| target == shot);
        if !overlaps_shot {
            self.shots_taken.push(target.clone());
            self.target = Cell { x: 4, y: 5 };
        }
        !overlaps_shot
    }

    fn ship_to_place_overlaps_any_locked(&self) -> bool {
        if let Some(ship) = self.ship_to_place() {
            Ship::any_overlap(&ship, &self.ships)
        } else {
            false
        }
    }

    fn lock_ship(&mut self) -> bool {
        let overlap = self.ship_to_place_overlaps_any_locked();
        if !overlap {
            if let Some(ship) = self.ship_to_place_mut() {
                ship.status = ShipStatus::Locked;
                return true;
            }
        }
        false
    }
}

// type alias for colors
type Color = [u8; 4];

// consts for colors
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

// frame size consts
pub const WIDTH: u32 = 720;
pub const HEIGHT: u32 = 600;
const TOP_MARGIN: usize = 190;
const GRID_WIDTH: usize = 301;
const GRID_MARGIN: usize = 40;
const CELL_WIDTH: usize = 30;
const CELL_MARGIN: usize = 4;
const CELL_COUNT: usize = 10;

impl World<'_> {
    /// Draw the `World` state to the frame buffer.
    pub fn draw(&self, frame: &mut [u8]) {
        let font = &self.font;
        World::clear_top(frame);
        World::clear_grids(frame);
        World::clear_bottom(frame);

        match self.status {
            GameStatus::Playing => {
                self.draw_ships(frame);
                self.draw_shots(frame);
                self.draw_target(frame);

                self.draw_info(frame, font);
            }
            GameStatus::End(_) => self.draw_end_message(frame),
        }
    }

    /// Create a new `World` instance with empty values
    pub fn new(game_type: GameType) -> Self {
        let font_data = include_bytes!("./source-code-pro-regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

        use GameType::*;
        let stream = match game_type {
            Ai => None,
            LocalNetwork => Some(try_connect().unwrap()),
        };

        World {
            this_player: Player::new(),
            other_player: Player::new(),
            status: GameStatus::Playing,
            font,
            stream,
            rng: thread_rng(),
        }
    }

    /// Update the `World` internal state
    pub fn update(&mut self, input: &WinitInputHelper) {
        use GameStatus::*;
        let moves = self.get_input_moves(input);
        match self.status {
            Playing => {
                use PlayerStatus::*;
                match self.this_player.status {
                    Placing => self.place_ships(&moves),
                    Aiming => World::aim(&moves, &mut self.this_player, &mut self.other_player),
                    Waiting => {
                        let other_moves = self.get_other_moves();
                        World::aim(&other_moves, &mut self.other_player, &mut self.this_player)
                    },
                }
                self.check_victory_condition();
            }
            End(_) => {
                self.wait_for_restart(&moves);
            }
        }
    }

    fn get_input_moves(&self, input: &WinitInputHelper) -> Vec<Move> {
        use Move::*;
        let mut moves = vec![];
        if input.key_pressed(VirtualKeyCode::Return) {
            moves.push(Enter);
        }
        if input.key_pressed(VirtualKeyCode::Space) {
            moves.push(Space);
        }
        if input.key_pressed(VirtualKeyCode::Up) {
            moves.push(Up);
        }
        if input.key_pressed(VirtualKeyCode::Down) {
            moves.push(Down);
        }
        if input.key_pressed(VirtualKeyCode::Right) {
            moves.push(Right);
        }
        if input.key_pressed(VirtualKeyCode::Left) {
            moves.push(Left);
        }
        moves
    }

    fn get_other_moves(&mut self) -> Vec<Move> {
        let mut moves = vec![];
        // move or shoot with some %
        // on average, move 10 times for every shot
        let shoot: f64 = self.rng.gen();
        match shoot {
            x if x > 0.1 => {
                let direction: Direction = self.rng.gen();
                moves.push(direction.into());
            }
            _ => {
                moves.push(Move::Enter);
            }
        }
        moves
    }

    fn begin_game(&mut self) {
        // for now, the game mode is: other player is a dumb AI
        // and places the ships exactly where you placed them
        self.other_player.ships = Ship::random_five(&mut self.rng);
        // for now, you have the first turn
        self.other_player.status = PlayerStatus::Waiting;
        self.this_player.status = PlayerStatus::Aiming;
    }

    fn clear_top(frame: &mut [u8]) {
        //
        // top frame pixels
        //
        for i in 0..TOP_MARGIN {
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
    }

    fn clear_grids(frame: &mut [u8]) {
        //
        // draw two grids
        //
        for line in 0..GRID_WIDTH {
            let i = (line + TOP_MARGIN) as usize;
            let w = WIDTH as usize;
            let grid_pixels: Vec<u8> = if line % CELL_WIDTH == 0 {
                repeat(GRID_LINES).take(GRID_WIDTH).flatten().collect()
            } else {
                // empty
                once(GRID_LINES)
                    .chain(repeat(GRID_EMPTY).take(29))
                    .cycle()
                    .take(GRID_WIDTH)
                    .flatten()
                    .collect()
            };

            let margin: Vec<u8> = repeat(BACKGROUND).take(GRID_MARGIN).flatten().collect();
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
    }

    fn clear_bottom(frame: &mut [u8]) {
        // 150px of empty (dark green)
        for i in TOP_MARGIN + GRID_WIDTH..HEIGHT as usize {
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
    }

    fn draw_ships(&self, frame: &mut [u8]) {
        for ship in self.this_player.ships.iter() {
            use ShipStatus::*;
            let color = match ship.status {
                Placing => YELLOW,
                Locked => GREEN,
                _ => GRID_EMPTY,
            };
            if ship.status != Hidden {
                for cell in &ship.cells {
                    World::fill_cell(cell, frame, color, true);
                }
            }
        }
    }

    fn draw_shots(&self, frame: &mut [u8]) {
        for shot in self.this_player.shots_taken.iter() {
            // check if it was a hit by iterating over the cells in the other player's ships
            let color = match self
                .other_player
                .ships
                .iter()
                .any(|ship| ship.cells.iter().any(|sc| sc == shot))
            {
                true => BLACK,  // hit
                false => WHITE, // miss
            };
            World::fill_cell(shot, frame, color, false);
        }

        for shot in self.other_player.shots_taken.iter() {
            // check if it was a hit by iterating over the cells in the other player's ships
            let color = match self
                .this_player
                .ships
                .iter()
                .any(|ship| ship.cells.iter().any(|sc| sc == shot))
            {
                true => BLACK,  // hit
                false => WHITE, // miss
            };
            World::fill_cell(shot, frame, color, true);
        }
    }

    fn draw_target(&self, frame: &mut [u8]) {
        if self.this_player.status == PlayerStatus::Aiming {
            World::fill_cell(&self.this_player.target, frame, FLAME, false);
        }

        if self.other_player.status == PlayerStatus::Aiming {
            World::fill_cell(&self.other_player.target, frame, YELLOW, true);
        }
    }

    fn draw_info(&self, frame: &mut [u8], font: &Font) {
        // title text
        World::draw_text(frame, "Battleship", font, GREEN, 60.0, (20.0, 0.0));

        match self.this_player.status {
            PlayerStatus::Placing => {
                World::draw_text(frame, "Place your ships!", font, WHITE, 40.0, (220.0, 60.0));
                let height = 24.0;
                for (i, text) in ["arrow keys to move", "space to rotate", "enter to place"]
                    .iter()
                    .enumerate()
                {
                    let y = 100.0 + i as f32 * height;
                    World::draw_text(frame, text, font, WHITE, height, (220.0, y));
                }
            }
            PlayerStatus::Aiming => {
                World::draw_text(frame, "Take aim!", font, WHITE, 40.0, (220.0, 60.0));
                let height = 24.0;
                for (i, text) in ["arrow keys to move", "space or enter to fire"]
                    .iter()
                    .enumerate()
                {
                    let y = 100.0 + i as f32 * height;
                    World::draw_text(frame, text, font, WHITE, height, (220.0, y));
                }
            }
            PlayerStatus::Waiting => {
                World::draw_text(
                    frame,
                    "Your opponent is aiming...",
                    font,
                    WHITE,
                    40.0,
                    (120.0, 90.0),
                );
            }
        }

        for i in 1..=10 {
            // grid numbering
            let offset = (
                (GRID_MARGIN + i * CELL_WIDTH - 20) as f32,
                TOP_MARGIN as f32 - 18.0,
            );
            World::draw_text(frame, &i.to_string(), font, WHITE, 18.0, offset);
            let offset = (
                (2 * GRID_MARGIN + GRID_WIDTH + i * CELL_WIDTH - 20) as f32,
                TOP_MARGIN as f32 - 18.0,
            );
            World::draw_text(frame, &i.to_string(), font, WHITE, 18.0, offset);

            let letter = ('A' as u8 + i as u8 - 1) as char;
            let offset = (
                (GRID_MARGIN - 18) as f32,
                (TOP_MARGIN + i * CELL_WIDTH - 22) as f32,
            );
            World::draw_text(frame, &letter.to_string(), font, WHITE, 18.0, offset);
            let offset = (
                (2 * GRID_MARGIN + GRID_WIDTH - 18) as f32,
                (TOP_MARGIN + i * CELL_WIDTH - 22) as f32,
            );
            World::draw_text(frame, &letter.to_string(), font, WHITE, 18.0, offset);
        }
    }

    fn draw_end_message(&self, frame: &mut [u8]) {
        match self.status {
            GameStatus::End(GameResult::Victory) => {
                World::draw_text(frame, "Glorious Victory!", &self.font, GREEN, 60.0, (120.0, 60.0));
                World::draw_text(
                    frame,
                    "press enter to restart",
                    &self.font,
                    WHITE,
                    40.0,
                    (120.0, 120.0),
                );
            }
            GameStatus::End(GameResult::Defeat) => {
                World::draw_text(
                    frame,
                    "Ignominious Defeat!",
                    &self.font,
                    FLAME,
                    60.0,
                    (120.0, 60.0),
                );
                World::draw_text(
                    frame,
                    "press enter to restart",
                    &self.font,
                    WHITE,
                    40.0,
                    (120.0, 120.0),
                );
            }
            _ => {},
        }
    }

    fn draw_text(
        frame: &mut [u8],
        text: &str,
        font: &Font,
        color: Color,
        height: f32,
        offset: (f32, f32),
    ) {
        let scale = Scale {
            x: height,
            y: height,
        };

        let v_metrics = font.v_metrics(scale);
        let offset = point(offset.0 + 0.0, offset.1 + v_metrics.ascent);

        let glyphs: Vec<_> = font.layout(text, scale, offset).collect();

        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    // Offset the position by the glyph bounding box
                    let x_offset = x + bounding_box.min.x as u32;
                    let y_offset = y + bounding_box.min.y as u32;
                    let index: usize = ((y_offset * WIDTH + x_offset) * 4) as usize;
                    // blend the colors
                    let blended_color = [
                        (BACKGROUND[0] as f32 * (1.0 - v) + color[0] as f32 * v) as u8,
                        (BACKGROUND[1] as f32 * (1.0 - v) + color[1] as f32 * v) as u8,
                        (BACKGROUND[2] as f32 * (1.0 - v) + color[2] as f32 * v) as u8,
                        0xff,
                    ];
                    frame[index..index + 4].copy_from_slice(&blended_color);
                });
            }
        }
    }

    fn fill_cell(cell: &Cell, frame: &mut [u8], color: Color, this_player: bool) {
        // cell width and height
        let filled_len = CELL_WIDTH - 2 * CELL_MARGIN;
        // one line _across_ within a filled cell
        let line: Vec<u8> = repeat(color).take(filled_len).flatten().collect();

        // whose cell is this?
        let grid_offset = if this_player {
            GRID_MARGIN
        } else {
            GRID_WIDTH + 2 * GRID_MARGIN
        };

        for i in 0..filled_len {
            let y_offset =
                (TOP_MARGIN + i + CELL_WIDTH * cell.y as usize + CELL_MARGIN) * 4 * WIDTH as usize;
            let x_offset = (grid_offset + CELL_WIDTH * cell.x as usize + CELL_MARGIN) * 4;
            let cell_start = y_offset + x_offset;
            frame[cell_start..cell_start + filled_len * 4].copy_from_slice(&line);
        }
    }

    fn wait_for_restart(&mut self, moves: &[Move]) {
        if moves.contains(&Move::Enter) {
            match self.stream {
                None => { *self = World::new(GameType::Ai) },
                Some(_) => { *self = World::new(GameType::LocalNetwork) }
            }
        }
    }

    fn aim(moves: &[Move], player: &mut Player, other_player: &mut Player) {
        for move_ in moves {
            use Move::*;
            match move_ {
                Down => { player.target.shift(&Direction::Down); },
                Up => { player.target.shift(&Direction::Up); },
                Right => { player.target.shift(&Direction::Right); },
                Left => { player.target.shift(&Direction::Left); },
                Enter | Space => {
                    if player.fire() {
                        player.status = PlayerStatus::Waiting;
                        other_player.status = PlayerStatus::Aiming;
                    }
                }
            }
        }
    }

    fn place_ships(&mut self, moves: &[Move]) {
        for _move in moves {
            use Move::*;
            match _move {
                Enter => {
                    if self.this_player.lock_ship() {
                        let next = self
                            .this_player
                            .ships
                            .iter_mut()
                            .find(|s| s.status == ShipStatus::Hidden);
                        match next {
                            Some(ship) => ship.placing(),
                            None => self.begin_game(),
                        }
                    }
                },
                Down => {
                    let ship = self.this_player.ship_to_place_mut().unwrap();
                    ship.shift(&Direction::Down);
                }
                Up => {
                    let ship = self.this_player.ship_to_place_mut().unwrap();
                    ship.shift(&Direction::Up);
                }
                Right => {
                    let ship = self.this_player.ship_to_place_mut().unwrap();
                    ship.shift(&Direction::Right);
                }
                Left => {
                    let ship = self.this_player.ship_to_place_mut().unwrap();
                    ship.shift(&Direction::Left);
                }
                Space => {
                    let ship = self.this_player.ship_to_place_mut().unwrap();
                    ship.rotate_right();
                }
            }
        }
    }

    fn check_victory_condition(&mut self) {
        // winning means all ships are sunk
        // so, for every cell in every ship, there's a shot from the other player that hits it
        let loss = self
            .this_player
            .ships
            .iter()
            .all(|ship| World::is_sunk(ship, &self.other_player.shots_taken));
        let win = self
            .other_player
            .ships
            .iter()
            .all(|ship| World::is_sunk(ship, &self.this_player.shots_taken));
        if loss {
            self.status = GameStatus::End(GameResult::Defeat);
        }
        if win {
            self.status = GameStatus::End(GameResult::Victory);
        }
    }

    fn is_sunk(ship: &Ship, shots: &[Cell]) -> bool {
        ship.cells.iter().all(|cell| shots.contains(cell))
    }
}
