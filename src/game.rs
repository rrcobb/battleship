use rand::prelude::Distribution;
use rand::{distributions::Standard, rngs::ThreadRng, thread_rng, Rng};
use rusttype::{point, Font, Scale};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::iter::{once, repeat};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

use crate::colors::*;
use crate::connection::*;

#[derive(Debug, Clone, PartialEq)]
enum GameResult {
    Victory,
    Defeat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameType {
    Ai,
    LocalNetwork,
}

#[derive(Debug, Clone, PartialEq)]
enum GameStatus {
    Starting,
    Playing(GameType),
    End(GameResult),
}

/// represents the settings ui, before the game has fully started
/// on game start, GameType becomes part of the GameStatus::Playing enum variant
struct Settings {
    game_type: GameType,
}

// 0-indexed grid positions
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct Cell {
    x: u8,
    y: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Action {
    Enter,
    Space,
    Right,
    Left,
    Up,
    Down,
}

impl From<Direction> for Action {
    fn from(item: Direction) -> Self {
        use Direction::*;
        match item {
            Up => Action::Up,
            Down => Action::Down,
            Right => Action::Right,
            Left => Action::Left,
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
        cell.extend_random_direction(rng, len)
    }

    fn extend_random_direction(&self, rng: &mut ThreadRng, len: &u8) -> Vec<Cell> {
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

    fn seq_from_origin(len: u8) -> Vec<Cell> {
        let cell = Cell { x: 0, y: 0 };
        cell.extend_down(len)
    }

    fn extend_down(&self, len: u8) -> Vec<Cell> {
        let mut cell = self.clone();
        let mut res = Vec::new();
        for _i in 0..len {
            res.push(cell.clone());
            cell.shift(&Direction::Down);
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
enum ShipStatus {
    #[default]
    Hidden,
    Placing,
    Locked,
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct Ship {
    status: ShipStatus,
    len: u8,
    cells: Vec<Cell>,
}

impl Ship {
    fn random_five(rng: &mut ThreadRng) -> Vec<Self> {
        use ShipStatus::*;
        let mut ships = Vec::new();
        for len in [2, 3, 4, 4, 5].iter() {
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
        let mut res: Vec<Self> = [2, 3, 4, 4, 5]
            .iter()
            .map(|&len| Ship {
                status: Hidden,
                len,
                cells: Cell::seq_from_origin(len),
            })
            .collect();
        res[0].status = Placing;
        res
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

    /// for the cell i
    /// move in the (x,y) direction i times
    fn rotate_right(&mut self) {
        // find the current direction:
        // difference between cells[0] and cells[1]
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
        use ShipStatus::*;
        self.ships.iter().find(|ship| ship.status == Placing)
    }

    fn ship_to_place_mut(&mut self) -> Option<&mut Ship> {
        use ShipStatus::*;
        self.ships.iter_mut().find(|ship| ship.status == Placing)
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

    fn lock_ship(&mut self) -> bool {
        let overlap = match self.ship_to_place() {
            Some(ship) => Ship::any_overlap(&ship, &self.ships),
            None => false
        };

        if !overlap {
            if let Some(ship) = self.ship_to_place_mut() {
                ship.status = ShipStatus::Locked;
                return true;
            }
        }
        false
    }
}

// frame size consts
pub const WIDTH: u32 = 720;
pub const HEIGHT: u32 = 600;
const TOP_MARGIN: usize = 190;
const GRID_WIDTH: usize = 301;
const GRID_MARGIN: usize = 40;
const CELL_WIDTH: usize = 30;
const CELL_MARGIN: usize = 4;
const CELL_COUNT: usize = 10;

/// Representation of the application state, plus some helpers (font, rng, tcp stream)
pub struct World<'a> {
    status: GameStatus,
    this_player: Player,
    other_player: Player,
    settings: Option<Settings>,
    font: Font<'a>,
    rng: ThreadRng,
    stream: Option<LinesCodec>,
}

impl World<'_> {
    /// render the `World` state to the frame buffer.
    pub fn render(&self, frame: &mut [u8]) {
        World::clear_top(frame);
        World::clear_grids(frame);
        World::clear_bottom(frame);

        match self.status {
            GameStatus::Starting => self.draw_start_screen(frame),
            GameStatus::Playing(_) => {
                self.draw_ships(frame);
                self.draw_shots(frame);
                self.draw_target(frame);
                self.draw_info(frame);
            }
            GameStatus::End(_) => self.draw_end_message(frame),
        }
    }

    /// Create a new `World` instance with empty values
    pub fn new() -> Self {
        let font_data = include_bytes!("../assets/source-code-pro-regular.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).unwrap();

        World {
            this_player: Player::new(),
            other_player: Player::new(),
            status: GameStatus::Starting,
            font,
            stream: None,
            rng: thread_rng(),
            settings: Some(Settings {
                game_type: GameType::LocalNetwork,
            }),
        }
    }

    /// Update the `World` internal state
    pub fn update(&mut self, input: &WinitInputHelper) {
        use GameStatus::*;
        let actions = self.get_input_actions(input);
        match self.status {
            Starting => {
                self.select_game_type(&actions);
            }
            Playing(_) => {
                use PlayerStatus::*;
                match self.this_player.status {
                    Placing => self.place_ships(&actions),
                    Aiming => {
                        World::aim(&actions, &mut self.this_player, &mut self.other_player);
                        self.broadcast_actions(&actions);
                    }
                    Waiting => {
                        let other_actions = self.get_other_actions();
                        World::aim(
                            &other_actions,
                            &mut self.other_player,
                            &mut self.this_player,
                        );
                    }
                }
                self.check_victory_condition();
            }
            End(_) => {
                self.wait_for_restart(&actions);
            }
        }
    }

    fn get_input_actions(&self, input: &WinitInputHelper) -> Vec<Action> {
        use Action::*;
        let mut actions = vec![];
        if input.key_pressed(VirtualKeyCode::Return) {
            actions.push(Enter);
        }
        if input.key_pressed(VirtualKeyCode::Space) {
            actions.push(Space);
        }
        if input.key_pressed(VirtualKeyCode::Up) {
            actions.push(Up);
        }
        if input.key_pressed(VirtualKeyCode::Down) {
            actions.push(Down);
        }
        if input.key_pressed(VirtualKeyCode::Right) {
            actions.push(Right);
        }
        if input.key_pressed(VirtualKeyCode::Left) {
            actions.push(Left);
        }
        actions
    }

    fn get_other_actions(&mut self) -> Vec<Action> {
        use GameStatus::*;
        use GameType::*;

        match self.status {
            Starting | End(_) => { unreachable!("should not be reading the other players actions unless we are Playing") }
            Playing(Ai) => self.gen_ai_actions(),
            Playing(LocalNetwork) => self.receive_broadcast_actions()
        }
    }

    fn broadcast_ship_positions(&mut self) {
        if self.stream.is_some() {
            let message = ron::ser::to_string(&self.this_player.ships).unwrap();
            match self.stream.as_mut().unwrap().send_message(&message) {
                Ok(_) => {}
                Err(e) => panic!("could not send message to connection, {}", e),
            };
        }
    }

    fn receive_ship_positions(&mut self) -> Vec<Ship> {
        if self.stream.is_some() {
            let actions_string = self.stream.as_mut().unwrap().read_message().unwrap();
            ron::de::from_str(&actions_string).unwrap()
        } else {
            panic!("no connection, cannot read ship positions.");
        }
    }

    fn receive_broadcast_actions(&mut self) -> Vec<Action> {
        if self.stream.is_some() {
            let actions_string = self.stream.as_mut().unwrap().read_message().unwrap();
            ron::de::from_str(&actions_string).unwrap()
        } else {
            panic!("no connection, cannot get the other player's moves.")
        }
    }

    fn broadcast_actions(&mut self, actions: &[Action]) {
        if self.stream.is_some() && !actions.is_empty() {
            let message = ron::ser::to_string(&actions).unwrap();
            match self.stream.as_mut().unwrap().send_message(&message) {
                Ok(_) => {}
                Err(e) => panic!("could not send message to connection, {}", e),
            }
        }
    }

    fn gen_ai_actions(&mut self) -> Vec<Action> {
        // move or shoot with some %
        // on average, move 10 times for every shot
        let shoot: f64 = self.rng.gen();
        let action = match shoot {
            x if x > 0.1 => {
                let direction: Direction = self.rng.gen();
                direction.into()
            }
            _ => Action::Enter,
        };
        vec![action]
    }

    fn begin_game(&mut self) {
        match self.status {
            GameStatus::Playing(GameType::Ai) => {
                self.other_player.ships = Ship::random_five(&mut self.rng);
                self.other_player.status = PlayerStatus::Waiting;
                self.this_player.status = PlayerStatus::Aiming;
            }
            GameStatus::Playing(GameType::LocalNetwork) => {
                self.broadcast_ship_positions();
                self.other_player.ships = self.receive_ship_positions();
                self.other_player.status = PlayerStatus::Waiting;
                self.this_player.status = PlayerStatus::Aiming;
            }
            _ => {}
        }
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

    fn draw_start_screen(&self, frame: &mut [u8]) {
        World::draw_text(frame, "Battleship", &self.font, GREEN, 60.0, (20.0, 0.0));
        if let Some(settings) = &self.settings {
            let (network_color, ai_color) = match settings.game_type {
                GameType::LocalNetwork => (YELLOW, GREEN),
                GameType::Ai => (GREEN, YELLOW),
            };
            World::draw_text(
                frame,
                "start local network game",
                &self.font,
                network_color,
                40.0,
                (40.0, 60.0),
            );
            World::draw_text(
                frame,
                "start game vs. computer",
                &self.font,
                ai_color,
                40.0,
                (40.0, 100.0),
            );

            let instructions = "up and down to select, enter to start";
            World::draw_text(frame, instructions, &self.font, WHITE, 22.0, (40.0, 150.0));
        }
    }

    fn draw_info(&self, frame: &mut [u8]) {
        let font = &self.font;
        // title text
        World::draw_text(frame, "Battleship", font, GREEN, 60.0, (20.0, 0.0));

        match self.this_player.status {
            PlayerStatus::Placing => {
                World::draw_text(frame, "Place your ships!", font, WHITE, 40.0, (200.0, 60.0));
                let height = 22.0;
                for (i, text) in ["arrow keys to move", "space to rotate", "enter to place"]
                    .iter()
                    .enumerate()
                {
                    let y = 100.0 + i as f32 * height;
                    World::draw_text(frame, text, font, WHITE, height, (250.0, y));
                }
            }
            PlayerStatus::Aiming => {
                World::draw_text(frame, "Take aim!", font, WHITE, 40.0, (200.0, 60.0));
                let height = 22.0;
                for (i, text) in ["arrow keys to move", "space or enter to fire"]
                    .iter()
                    .enumerate()
                {
                    let y = 100.0 + i as f32 * height;
                    World::draw_text(frame, text, font, GREEN, height, (250.0, y));
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
                World::draw_text(
                    frame,
                    "Glorious Victory!",
                    &self.font,
                    GREEN,
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
            _ => {}
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

    fn wait_for_restart(&mut self, actions: &[Action]) {
        if actions.contains(&Action::Enter) {
            // TODO: restart with existing stream if connected to an opponent
            *self = World::new()
        }
    }

    fn aim(actions: &[Action], player: &mut Player, other_player: &mut Player) {
        for action in actions {
            use Action::*;
            match action {
                Down => {
                    player.target.shift(&Direction::Down);
                }
                Up => {
                    player.target.shift(&Direction::Up);
                }
                Right => {
                    player.target.shift(&Direction::Right);
                }
                Left => {
                    player.target.shift(&Direction::Left);
                }
                Enter | Space => {
                    if player.fire() {
                        player.status = PlayerStatus::Waiting;
                        other_player.status = PlayerStatus::Aiming;
                    }
                }
            }
        }
    }

    fn place_ships(&mut self, actions: &[Action]) {
        for action in actions {
            use Action::*;
            match action {
                Enter => {
                    if self.this_player.lock_ship() {
                        let next = self
                            .this_player
                            .ships
                            .iter_mut()
                            .find(|s| s.status == ShipStatus::Hidden);
                        match next {
                            Some(ship) => ship.status = ShipStatus::Placing,
                            None => self.begin_game(),
                        }
                    }
                }
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

    fn select_game_type(&mut self, actions: &[Action]) {
        use Action::*;
        use GameType::*;
        for _action in actions {
            match _action {
                Up | Down => {
                    let game_type = match &self.settings {
                        None => LocalNetwork,
                        Some(s) if s.game_type == Ai => LocalNetwork,
                        Some(_) => Ai,
                    };
                    self.settings = Some(Settings { game_type });
                }
                Enter => {
                    if let Some(settings) = &self.settings {
                        self.stream = match settings.game_type {
                            Ai => None,
                            LocalNetwork => Some(try_connect().unwrap()),
                        };
                        self.status = GameStatus::Playing(settings.game_type);
                    }
                }
                _ => {}
            }
        }
    }

    /// winning means all ships are sunk
    /// so, for every cell in every ship, there's a shot from the other player that hits it
    fn check_victory_condition(&mut self) {
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
