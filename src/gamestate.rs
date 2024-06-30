// https://www.w3.org/2001/06/utf-8-test/UTF-8-demo.html

use std::cmp::{max, min};

use rand::Rng;
use ruscii::{drawing::Pencil, keyboard::Key, spatial::Vec2, terminal::Color};

use crate::cell::Cell;

const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 10;

const GAMEOVER_WIDTH: i32 = 11;
const GAMEOVER_HEIGHT: i32 = 3;

const NB_BOMBS: i32 = 20;

#[derive(PartialEq)]
enum RunningState {
    Running,
    GameOver,
}

static AROUND_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub struct GameState {
    dimension: Vec2,
    step: usize,
    grid: [[Cell; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
    grid_pos: Vec2,
    running: RunningState,
    pub prev_key: Option<Key>,
    cursor: Vec2,
    bombs: [Vec2; NB_BOMBS as usize],
    // gameover
    gameover_pos: Vec2,
    gameover_speed: Vec2,
}

impl GameState {
    pub fn new(dim: Vec2) -> Self {
        Self {
            dimension: dim,
            step: 0,
            grid: [[Cell::Hidden; GRID_WIDTH as usize]; GRID_HEIGHT as usize],
            grid_pos: Vec2::xy((dim.x - GRID_WIDTH * 5) / 2, (dim.y - GRID_HEIGHT * 3) / 2),
            bombs: [Vec2::xy(-1, -1); NB_BOMBS as usize],
            cursor: Vec2::xy(0, 0),
            running: RunningState::Running,
            prev_key: None,
            gameover_pos: Vec2::zero(),
            gameover_speed: Vec2::zero(),
        }
    }

    pub fn init(&mut self) {
        for i in 0..NB_BOMBS {
            self.bombs[i as usize] = self.find_empty_bomb_pos();
        }
    }

    fn find_empty_bomb_pos(&self) -> Vec2 {
        let pos = Vec2::xy(
            rand::thread_rng().gen_range(0..GRID_WIDTH),
            rand::thread_rng().gen_range(0..GRID_HEIGHT),
        );
        if self.is_on_bomb(&pos) {
            return self.find_empty_bomb_pos();
        }
        pos
    }

    pub fn set_step(&mut self, step: usize) {
        self.step = step;
    }

    pub fn handle_keys_down(&mut self, keys_down: Vec<Key>) {
        if keys_down.is_empty() {
            self.prev_key = None;
        } else {
            for key_down in keys_down {
                self.handle_key_down(key_down);
            }
        }
    }

    fn handle_key_down(&mut self, key_down: Key) {
        if Some(key_down) == self.prev_key {
            match key_down {
                // don't repeat these
                Key::Space | Key::F | Key::Enter => {}
                // everything else, we just slow down the repeat
                _ => self.prev_key = None,
            }
            return;
        }

        self.prev_key = None;
        match key_down {
            Key::Left => self.move_cursor(Vec2::xy(-1, 0)),
            Key::Right => self.move_cursor(Vec2::xy(1, 0)),
            Key::Up => self.move_cursor(Vec2::xy(0, -1)),
            Key::Down => self.move_cursor(Vec2::xy(0, 1)),
            Key::Space => self.reveal(),
            Key::F | Key::Enter => self.flag(),
            _ => (),
        }
        self.prev_key = Some(key_down);
    }

    pub fn update(&mut self) {
        if self.running == RunningState::GameOver {
            if (self.step as i32) % 4 == 0 {
                self.gameover_pos += self.gameover_speed;
                if self.gameover_pos.x + GAMEOVER_WIDTH == GRID_WIDTH || self.gameover_pos.x == 0 {
                    self.gameover_speed.x = -self.gameover_speed.x
                }
                if self.gameover_pos.y + GAMEOVER_HEIGHT == GRID_HEIGHT || self.gameover_pos.y == 0
                {
                    self.gameover_speed.y = -self.gameover_speed.y
                }
            }
            return;
        }
    }

    fn gameover(&mut self) {
        self.running = RunningState::GameOver;
        self.gameover_pos = Vec2::xy((GRID_WIDTH - GAMEOVER_WIDTH) / 2, 2);
        self.gameover_speed = Vec2::xy(1, 1);
    }

    pub fn draw(&mut self, pencil: &mut Pencil) {
        match self.running {
            RunningState::Running => self.draw_running(pencil),
            RunningState::GameOver => self.draw_gameover(pencil),
        }
    }

    fn draw_gameover(&mut self, pencil: &mut Pencil) {
        let vd = Vec2::xy(0, 1); // down
        let vr = Vec2::xy(1, 0); // right
        self.draw_running(pencil);

        // bombs
        self.bombs.iter().for_each(|&pos| {
            let pos = self.tx_to_grid(pos.x, pos.y);
            pencil
                .set_background(Color::Red)
                .draw_text(" ╲ ╱ ", pos - vd)
                .draw_text("  ╳  ", pos)
                .draw_text(" ╱ ╲ ", pos + vd);
        });

        // gameover box
        pencil
            .set_foreground(Color::Xterm(230))
            .set_background(Color::Xterm(100));
        let Vec2 { x, y } = self.gameover_pos;
        pencil.draw_text("           ", self.tx_to_grid(x, y) - vd);
        pencil.draw_text(" GAME OVER ", self.tx_to_grid(x, y));
        pencil.draw_text("           ", self.tx_to_grid(x, y) + vd);
    }

    fn draw_running(&mut self, pencil: &mut Pencil) {
        let vd = Vec2::xy(0, 1); // down
        let vr = Vec2::xy(1, 0); // right
                                 // instructions
        let mut y = 0;
        pencil.set_foreground(Color::White);
        pencil.draw_text(&format!("arrow keys: move"), self.tx_to_grid(-5, y));
        y += 1;
        pencil.draw_text(&format!("space: reveal"), self.tx_to_grid(-5, y));
        y += 1;
        pencil.draw_text(&format!("enter/f: flag"), self.tx_to_grid(-5, y));
        y += 2;
        pencil.draw_text(&format!("q/esc: quit"), self.tx_to_grid(-5, y));

        // score
        y = 0;
        let mut nb_flags = 0;
        for row in self.grid.iter() {
            for cell in row.iter() {
                if *cell == Cell::Flag {
                    nb_flags += 1;
                }
            }
        }

        pencil.draw_text(
            &format!("bombs: {}", NB_BOMBS - nb_flags),
            self.tx_to_grid(GRID_WIDTH + 1, y),
        );

        // draw border
        pencil
            .set_background(Color::Black)
            .set_foreground(Color::Xterm(250));
        pencil.draw_vline('│', self.tx_to_grid(0, 0) - vd - vr, GRID_HEIGHT * 3);
        pencil.draw_vline('│', self.tx_to_grid(GRID_WIDTH, 0) - vd, GRID_HEIGHT * 3);
        pencil.draw_hline('─', self.tx_to_grid(0, -1) + vd, GRID_WIDTH * 5);
        pencil.draw_hline('─', self.tx_to_grid(0, GRID_HEIGHT) - vd, GRID_WIDTH * 5);
        pencil.draw_text("╰", self.tx_to_grid(0, GRID_HEIGHT) - vd - vr);
        pencil.draw_text("╯", self.tx_to_grid(GRID_WIDTH, GRID_HEIGHT) - vd);
        pencil.draw_text("╭", self.tx_to_grid(0, -1) + vd - vr);
        pencil.draw_text("╮", self.tx_to_grid(GRID_WIDTH, -1) + vd);

        // draw grid
        for (y, row) in self.grid.iter().enumerate() {
            let y = y as i32;
            for (x, cell) in row.iter().enumerate() {
                let x = x as i32;
                let pos = self.tx_to_grid(x, y);

                let bg = if self.cursor.x == x && self.cursor.y == y {
                    Color::Xterm(234)
                } else {
                    Color::Black
                };
                match cell {
                    Cell::Hidden => {
                        // not revealed yet
                        _ = pencil
                            .set_background(bg)
                            .set_foreground(Color::LightGrey)
                            .draw_text(&format!("     "), pos - vd)
                            .draw_text("  ∙  ", pos)
                            .draw_text(&format!("     "), pos + vd);
                        // +                            .draw_text(&format!("▇▇▇▇▇▇"), pos - vd)
                        // +                            .draw_text("████▉", pos)
                        // +                            .draw_text(&format!("████▉"), pos + vd);
                    }
                    Cell::Flag => {
                        // flag
                        _ = pencil
                            .set_background(bg)
                            .set_foreground(Color::Red)
                            .draw_text(&format!("╔═══╗"), pos - vd)
                            .draw_text("║ ╳ ║", pos)
                            .draw_text(&format!("╚═══╝"), pos + vd);
                    }
                    Cell::Revealed(nb) => {
                        _ = {
                            if *nb == 0 {
                                // empty
                                pencil
                                    .set_background(bg)
                                    .set_foreground(Color::LightGrey)
                                    .draw_text("     ", pos - vd)
                                    .draw_text("     ", pos)
                                    .draw_text("     ", pos + vd);
                            } else {
                                let color = match nb {
                                    1 => Color::Blue,
                                    2 => Color::Green,
                                    3 => Color::Red,
                                    4 => Color::Magenta,
                                    5 => Color::Yellow,
                                    6 => Color::Cyan,
                                    7 => Color::White,
                                    8 => Color::Grey,
                                    _ => Color::Grey,
                                };
                                // number
                                pencil
                                    .set_background(bg)
                                    .set_foreground(color)
                                    .draw_text(&format!("╭───╮"), pos - vd)
                                    .draw_text(&format!("│ {} │", nb), pos)
                                    .draw_text(&format!("╰───╯"), pos + vd);
                            }
                        }
                    }
                };
            }
        }
    }

    //--------------------------------------------------------------------------------
    // helpers
    //--------------------------------------------------------------------------------

    fn nb_bombs_on_pos(&mut self, pos: &Vec2) -> usize {
        let x = pos.x as i32;
        let y = pos.y as i32;
        AROUND_OFFSETS
            .iter()
            .map(|(dx, dy)| Vec2::xy(x + dx, y + dy))
            .filter(|&pos| self.is_in_grid(&pos))
            .filter(|&pos| self.is_on_bomb(&pos))
            .count()
    }

    fn flag(&mut self) {
        match self.grid[self.cursor.y as usize][self.cursor.x as usize] {
            Cell::Hidden => self.grid[self.cursor.y as usize][self.cursor.x as usize] = Cell::Flag,
            Cell::Revealed(_) => return,
            Cell::Flag => self.grid[self.cursor.y as usize][self.cursor.x as usize] = Cell::Hidden,
        }
    }

    fn reveal(&mut self) {
        if self.is_on_bomb(&self.cursor) {
            // we clicked on a bomb...
            self.gameover();
            return;
        }
        if self.grid[self.cursor.y as usize][self.cursor.x as usize] != Cell::Hidden {
            // cell was already revealed
            return;
        }
        let mut cells_to_check = vec![self.cursor];
        while let Some(pos) = cells_to_check.pop() {
            let nb_bombs = self.nb_bombs_on_pos(&pos);
            self.grid[pos.y as usize][pos.x as usize] = Cell::Revealed(nb_bombs);
            if nb_bombs == 0 {
                let x = pos.x as i32;
                let y = pos.y as i32;
                // add all surrounding cells to list
                AROUND_OFFSETS
                    .iter()
                    .map(|(dx, dy)| Vec2::xy(x + dx, y + dy))
                    .filter(|&pos| self.is_in_grid(&pos))
                    .filter(|&pos| self.grid[pos.y as usize][pos.x as usize] == Cell::Hidden)
                    .for_each(|pos| cells_to_check.push(pos))
            }
        }
    }

    fn move_cursor(&mut self, delta: Vec2) {
        self.cursor += delta;
        self.cursor.x = max(0, self.cursor.x);
        self.cursor.y = max(0, self.cursor.y);
        self.cursor.x = min(GRID_WIDTH - 1, self.cursor.x);
        self.cursor.y = min(GRID_HEIGHT - 1, self.cursor.y);
    }

    fn tx_to_grid(&self, x: i32, y: i32) -> Vec2 {
        return Vec2::xy(x * 5 + self.grid_pos.x, y * 3 + self.grid_pos.y);
    }

    fn is_in_grid(&self, pos: &Vec2) -> bool {
        pos.x >= 0 && pos.x < GRID_WIDTH && pos.y >= 0 && pos.y < GRID_HEIGHT
    }

    fn is_on_bomb(&self, pos: &Vec2) -> bool {
        self.bombs.contains(pos)
    }
}
