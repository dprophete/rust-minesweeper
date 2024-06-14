use std::cmp::{max, min};

use ruscii::{drawing::Pencil, keyboard::Key, spatial::Vec2, terminal::Color};

use crate::tetromino::Tetromino;

const GRID_WIDTH: i32 = 10;
const GRID_HEIGHT: i32 = 20;

#[derive(PartialEq)]
enum State {
    Running,
    GameOver,
}

pub struct GameState {
    dimension: Vec2,
    grid: [[Cell; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
    gpos: Vec2,
    current_piece: Option<Piece>,
    pub prev_key: Option<Key>,
    running: State,
    drop_current_piece: bool,
    lines_cleared: i32,
    nb_next_pieces: i32,
    next_pieces: Vec<Tetromino>,
}

impl GameState {
    pub fn new(dim: Vec2) -> Self {
        Self {
            dimension: dim,
            grid: [[Cell::Empty; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
            gpos: Vec2::xy((dim.x - GRID_WIDTH * 2) / 2, (dim.y - GRID_HEIGHT) / 2),
            current_piece: None,
            prev_key: None,
            running: State::Running,
            drop_current_piece: false,
            lines_cleared: 0,
            nb_next_pieces: 3,
            next_pieces: vec![],
        }
    }

    pub fn init(&mut self) {
        for _ in 0..self.nb_next_pieces {
            self.next_pieces.push(Tetromino::random());
        }

        // self.init_with_all_pieces();
        self.pick_current_piece();
    }

    pub fn handle_keys_down(&mut self, keys_down: Vec<Key>, step: usize) {
        if keys_down.is_empty() {
            self.prev_key = None;
        } else {
            for key_down in keys_down {
                self.handle_key_down(key_down, step);
            }
        }
    }
    fn handle_key_down(&mut self, key_down: Key, _step: usize) {
        if Some(key_down) == self.prev_key {
            match key_down {
                // don't repeat these
                Key::Up | Key::Space | Key::Down | Key::Enter => {}
                // everything else, we just slow down the repeat
                _ => self.prev_key = None,
            }
            return;
        }

        match key_down {
            Key::A => self.move_gpos(Vec2::xy(-2, 0)),
            Key::D => self.move_gpos(Vec2::xy(2, 0)),
            Key::W => self.move_gpos(Vec2::xy(0, -1)),
            Key::S => self.move_gpos(Vec2::xy(0, 1)),
            Key::Up | Key::Space => _ = self.rotate_current_piece(),
            Key::Down | Key::Enter => _ = self.drop_current_piece(),
            Key::Left => _ = self.move_current_piece(Vec2::xy(-1, 0)),
            Key::Right => _ = self.move_current_piece(Vec2::xy(1, 0)),
            _ => (),
        }
        self.prev_key = Some(key_down);
    }

    pub fn update(&mut self, step: usize) {
        if self.running == State::GameOver {
            return;
        }

        // moving current piece
        if self.current_piece.is_some() {
            // remove shadow
            for y in 0..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    if self.grid[y as usize][x as usize] == Cell::Shadow {
                        self.grid[y as usize][x as usize] = Cell::Empty;
                    }
                }
            }

            let mut draw_current = true;
            let step_delay = if self.drop_current_piece { 1 } else { 10 };
            if step % step_delay == 0 {
                draw_current = self.move_current_piece(Vec2::xy(0, 1));
            }

            if draw_current {
                // piece was able to go down
                let current_piece = self.current_piece.unwrap();

                // draw shadow
                self.remove_piece(&current_piece);
                let mut shadow_piece = current_piece.clone();

                while self.is_piece_in_grid(&shadow_piece)
                    && self.is_piece_in_empty_pos(&shadow_piece)
                {
                    shadow_piece.pos.y = shadow_piece.pos.y + 1;
                }
                shadow_piece.pos.y = shadow_piece.pos.y - 1;
                self.place_shadow(&shadow_piece);

                self.place_piece(&current_piece);
            } else {
                // piece reached the bottom
                let current_piece = self.current_piece.unwrap();
                self.place_piece(&current_piece);

                // check if we have a full rows
                for y in 0..GRID_HEIGHT {
                    if self.is_row_full(y) {
                        // move all rows above one row down
                        self.lines_cleared += 1;
                        for y2 in (0..y).rev() {
                            self.copy_row_down(y2);
                        }

                        // clear top row
                        self.clear_row(0);
                    }
                }

                // pick a new piece
                self.pick_current_piece();
                // TODO: check if game over
                self.place_piece(&self.current_piece.unwrap());
            }
        }
    }

    pub fn draw(&mut self, pencil: &mut Pencil, _step: usize) {
        // score
        pencil.set_foreground(Color::White);
        pencil.set_foreground(Color::White).draw_text(
            &format!("lines cleared: {}", self.lines_cleared),
            self.tx_to_grid(GRID_WIDTH * 2 + 4, 0),
        );
        pencil
            .set_foreground(Color::White)
            .draw_text("next pieces:", self.tx_to_grid(GRID_WIDTH * 2 + 4, 2));

        for (i, tetromino) in self.next_pieces.clone().iter().enumerate() {
            let mut piece = Piece::new(*tetromino);
            piece.pos = self.tx_to_grid(GRID_WIDTH * 2 + 4, 4 + i as i32 * 5);
            self.draw_piece(pencil, &piece);
        }

        // draw border
        pencil
            .set_background(Color::Black)
            .set_foreground(Color::White);
        pencil.draw_vline('|', self.tx_to_grid(-1, 0), GRID_HEIGHT);
        pencil.draw_vline('|', self.tx_to_grid(GRID_WIDTH * 2, 0), GRID_HEIGHT);
        pencil.draw_hline('-', self.tx_to_grid(0, GRID_HEIGHT), GRID_WIDTH * 2);
        pencil.draw_text("+", self.tx_to_grid(-1, GRID_HEIGHT));
        pencil.draw_text("+", self.tx_to_grid(GRID_WIDTH * 2, GRID_HEIGHT));

        // draw grid
        pencil.set_foreground(Color::LightGrey);
        for (y, row) in self.grid.iter().enumerate() {
            let y = y as i32;
            for (x, cell) in row.iter().enumerate() {
                let x = x as i32 * 2;
                let pos = self.tx_to_grid(x, y);
                match cell {
                    Cell::Empty => pencil.set_background(Color::Black).draw_text("∙∙", pos),
                    Cell::Tetromino(tetromino) => pencil
                        .set_background(tetromino.color())
                        .draw_text("  ", pos),
                    Cell::Shadow => pencil.set_background(Color::DarkGrey).draw_text("∙∙", pos),
                };
            }
        }
    }

    //--------------------------------------------------------------------------------
    // helpers
    //--------------------------------------------------------------------------------

    fn tx_to_grid(&self, x: i32, y: i32) -> Vec2 {
        return Vec2::xy(x + self.gpos.x, y + self.gpos.y);
    }

    fn is_in_grid(&self, pos: Vec2) -> bool {
        pos.x >= 0 && pos.x < GRID_WIDTH && pos.y >= 0 && pos.y < GRID_HEIGHT
    }

    fn is_in_empty_pos(&self, pos: Vec2) -> bool {
        let cell = self.grid[pos.y as usize][pos.x as usize];
        cell == Cell::Empty || cell == Cell::Shadow
    }

    fn copy_row_down(&mut self, row: i32) {
        self.grid[(row + 1) as usize] = self.grid[row as usize].clone();
    }

    fn clear_row(&mut self, row: i32) {
        for x in 0..GRID_WIDTH {
            self.grid[row as usize][x as usize] = Cell::Empty;
        }
    }

    fn is_row_full(&self, row: i32) -> bool {
        for x in 0..GRID_WIDTH {
            if self.grid[row as usize][x as usize] == Cell::Empty {
                return false;
            }
        }
        true
    }

    fn is_piece_in_empty_pos(&self, piece: &Piece) -> bool {
        for cell in piece.cells().iter() {
            if !self.is_in_empty_pos(piece.pos + *cell) {
                return false;
            }
        }
        true
    }

    fn is_piece_in_grid(&self, piece: &Piece) -> bool {
        for cell in piece.cells().iter() {
            if !self.is_in_grid(piece.pos + *cell) {
                return false;
            }
        }
        true
    }

    pub fn pick_current_piece(&mut self) {
        let tetromino = self.next_pieces.remove(0);
        self.next_pieces.push(Tetromino::random());
        let mut piece = Piece::new(tetromino);
        piece.pos = Vec2::xy(GRID_WIDTH / 2 - 1, 0);
        self.current_piece = Some(piece);
        self.drop_current_piece = false;
    }

    fn place_piece(&mut self, piece: &Piece) {
        for cell in piece.cells().iter() {
            let x = piece.pos.x + cell.x;
            let y = piece.pos.y + cell.y;
            if x >= 0 && x < GRID_WIDTH && y >= 0 && y < GRID_HEIGHT {
                self.grid[y as usize][x as usize] = Cell::Tetromino(piece.tetromino);
            }
        }
    }

    fn draw_piece(&mut self, pencil: &mut Pencil, piece: &Piece) {
        pencil.set_background(piece.tetromino.color());
        for cell in piece.cells().iter() {
            let x = piece.pos.x + cell.x * 2;
            let y = piece.pos.y + cell.y;
            pencil.draw_text("  ", Vec2::xy(x, y));
        }
    }

    fn place_shadow(&mut self, piece: &Piece) {
        for cell in piece.cells().iter() {
            let x = piece.pos.x + cell.x;
            let y = piece.pos.y + cell.y;
            if x >= 0 && x < GRID_WIDTH && y >= 0 && y < GRID_HEIGHT {
                self.grid[y as usize][x as usize] = Cell::Shadow;
            }
        }
    }

    fn remove_piece(&mut self, piece: &Piece) {
        for cell in piece.cells().iter() {
            let x = piece.pos.x + cell.x;
            let y = piece.pos.y + cell.y;
            if x >= 0 && x < GRID_WIDTH && y >= 0 && y < GRID_HEIGHT {
                self.grid[y as usize][x as usize] = Cell::Empty;
            }
        }
    }

    fn move_gpos(&mut self, delta: Vec2) {
        self.gpos += delta;
        self.gpos.x = max(0, self.gpos.x);
        self.gpos.y = max(0, self.gpos.y);
        self.gpos.x = min(self.dimension.x - GRID_WIDTH * 2, self.gpos.x);
        self.gpos.y = min(self.dimension.y - GRID_WIDTH * 2, self.gpos.y);
    }

    fn drop_current_piece(&mut self) {
        self.drop_current_piece = true;
    }

    fn move_current_piece(&mut self, delta: Vec2) -> bool {
        if let Some(piece) = self.current_piece {
            // let's first make sure we remove the current piece from the grid
            self.remove_piece(&piece);

            let mut new_piece = piece.clone();
            new_piece.pos += delta;
            if self.is_piece_in_grid(&new_piece) && self.is_piece_in_empty_pos(&new_piece) {
                self.current_piece = Some(new_piece);
                return true;
            }
        }
        return false;
    }

    fn rotate_current_piece(&mut self) -> bool {
        if let Some(piece) = self.current_piece {
            // let's first make sure we remove the current piece from the grid
            self.remove_piece(&piece);

            let mut new_piece = piece.clone();
            new_piece.rotate(1);

            if self.is_piece_in_grid(&new_piece) && self.is_piece_in_empty_pos(&new_piece) {
                self.current_piece = Some(new_piece);
                return true;
            }
        }
        return false;
    }

    fn init_with_all_pieces(&mut self) {
        for (t_nb, t) in [
            Tetromino::I,
            Tetromino::J,
            Tetromino::L,
            Tetromino::O,
            Tetromino::S,
            Tetromino::T,
            Tetromino::Z,
        ]
        .iter()
        .enumerate()
        {
            for rot in 0..=3 {
                let mut piece = Piece::new(*t);
                piece.pos = Vec2::xy(rot * 5, t_nb as i32 * 5);
                piece.rotate(rot);
                self.place_piece(&piece);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub tetromino: Tetromino,
    pub rot: i32,
    pub pos: Vec2,
}

impl Piece {
    pub fn new(tetromino: Tetromino) -> Self {
        Self {
            tetromino,
            rot: 0,
            pos: Vec2::zero(),
        }
    }

    pub fn rotate(&mut self, delta: i32) -> &Self {
        self.rot = (self.rot + delta) % 4;
        self
    }

    pub fn cells(&self) -> Vec<Vec2> {
        self.tetromino.cells(self.rot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Tetromino(Tetromino),
    Shadow,
}
