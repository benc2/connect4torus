use crate::cell::Cell;
use crate::Player;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew::prelude::*;

fn modulo(n: i32, m: i32) -> i32 {
    // always returns in range [0,m)
    (n % m + m) % m
}

#[derive(Debug, Clone)]
pub struct InsertError;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Properties)]
pub struct Board {
    pub board: Vec<Vec<Option<Player>>>,
    pub width: u8,
    pub height: u8,
}

impl Board {
    pub fn new(width: u8, height: u8) -> Self {
        let mut empty_col = Vec::new();
        for _ in 0..height {
            empty_col.push(None);
        }
        let mut board = Vec::new();
        for _ in 0..width {
            board.push(empty_col.clone());
        }
        Board {
            board,
            width,
            height,
        }
    }

    pub fn insert(&mut self, column: usize, player: &Player) -> Result<usize, InsertError> {
        for row in 0..self.height {
            if self.board[column][row as usize].is_none() {
                self.board[column][row as usize] = Some(player.clone());
                return Ok(row.into());
            }
        }
        Err(InsertError)
    }

    #[allow(dead_code)] //TODO: remove function if not necessary
    fn column_full(&self, colnr: usize) -> bool {
        self.board[colnr].last().is_some()
    }

    fn shift_coords(
        &self,
        (col, row): (usize, usize),
        (dx, dy): (i32, i32),
        amount: i32,
    ) -> (usize, usize) {
        let pre_x = modulo(col as i32 + amount * dx, self.width as i32);
        let pre_y = modulo(row as i32 + amount * dy, self.height as i32);
        // log::info!("{}", format!("{} {}", pre_x, pre_y));
        let x: usize = pre_x.try_into().unwrap();
        let y: usize = pre_y.try_into().unwrap();
        (x, y)
    }

    fn check_line(
        &self,
        col: usize,
        row: usize,
        dx: i32,
        dy: i32,
        player: &Player,
        win_length: usize,
    ) -> bool {
        // Note that this does not check for duplicates in the same line
        // i.e. if the board is smaller than the win_length in some direction
        // filling up that direction with one color will be a win even though it's shorter than win_length
        let mut consecutive = 0;
        for s in -(win_length as i32)..(win_length as i32) {
            let (x, y) = self.shift_coords((col, row), (dx, dy), s as i32);
            let cell_status = self.board[x][y].clone(); // why clone?
            if cell_status == Some(player.clone()) {
                consecutive += 1;
            } else {
                consecutive = 0;
            }

            if consecutive >= win_length {
                return true;
            }
        }
        false
    }

    pub fn check_win(
        // returns bool. If needed, switch back to returning winning direction if present
        &self,
        col: usize,
        row: usize,
        player: &Player,
        win_length: usize,
    ) -> bool {
        let directions = vec![(1, 1), (1, 0), (1, -1), (0, -1)];
        for (dx, dy) in directions {
            if self.check_line(col, row, dx, dy, player, win_length) {
                return true;
            }
            // log::info!("Checked line!");
        }
        false
    }

    fn find_adjacent_chips_in_dir(
        &self,
        col: usize,
        row: usize,
        direction: (i32, i32),
        player: &Player,
    ) -> HashSet<(usize, usize)> {
        let mut found_positions = HashSet::new();
        let mut s = 0;
        loop {
            // first find all consecutive
            let (x, y) = self.shift_coords((col, row), direction, s);
            if found_positions.contains(&(x, y)) {
                // if we loop all the way around, we can return
                // TODO: this actually isn't necessary: this situation only occurs when a full "loop"
                // (aka row/column/snake through diagonals) is filled, however, if that loop is only missing 1 spot,
                // there is already a winning connect line as long as every loop is at least one longer than the win-length
                // (having the width and height > win_length guarantees this is the case), hence you can never complete the
                // loop
                // Keeping the HashSet is still useful though, since we use it to look up if a chip is winning
                return found_positions;
            }
            if self.board[x][y].as_ref() == Some(player) {
                found_positions.insert((x, y));
            } else {
                break; // consecutive chain broken
            }
            s += 1;
        }
        s = -1;
        loop {
            let (x, y) = self.shift_coords((col, row), direction, s);
            // since we did not return yet, we know there is one opponent chip/empty spot in the line
            // so we don't need to check whether we loop around
            if self.board[x][y].as_ref() == Some(player) {
                found_positions.insert((x, y));
            } else {
                break;
            }
            s -= 1;
        }
        found_positions
    }

    pub fn find_winning_chips(
        &self,
        col: usize,
        row: usize,
        player: &Player,
        win_length: usize,
    ) -> HashSet<(usize, usize)> {
        let directions = vec![(1, 1), (1, 0), (1, -1), (0, -1)]; // TODO copied code
        let mut found_positions = HashSet::new();
        for dir in directions {
            let adj_chips_in_dir = self.find_adjacent_chips_in_dir(col, row, dir, player);
            if adj_chips_in_dir.len() >= win_length {
                // in this case the adjacent chips are a winning line
                // TODO note that now the check_line and check win methods are essentially redundant,
                // (except for being somewhat more efficient?)
                found_positions.extend(self.find_adjacent_chips_in_dir(col, row, dir, player));
            }
        }
        found_positions
    }
}

#[derive(PartialEq, Properties)]
pub struct BoardProps {
    pub board: Board,
    pub winning_chips: Option<HashSet<(usize, usize)>>,
    pub column_callbacks: Vec<Callback<MouseEvent>>, // define in gamedata component as  ctx.link().callback(move |_| Msg::ColumnClick(colnr))
}

#[function_component(BoardView)]
pub fn board_view(boardprops: &BoardProps) -> Html {
    let board_html = boardprops
        .board
        .board
        .iter()
        .enumerate()
        .map(|(colnr, column)| {
            // let mut counter = 0;
            // let mut columnstr = String::new();
            let mut column_cells = html! {};
            for (row, cell_status) in column.iter().enumerate() {
                let winning;
                if boardprops.winning_chips.is_some() {
                    winning = (&boardprops.winning_chips)
                        .as_ref()
                        .unwrap()
                        .contains(&(colnr, row));
                } else {
                    winning = false;
                }

                column_cells = html! { // prepend new cell to existing html
                    <>
                        <Cell status={cell_status.clone()} winning={winning}/>
                        {column_cells}
                    </>
                }
            }
            // let columnstr = "kaas";
            let on_column_click = &boardprops.column_callbacks[colnr];
            html! {<button class="column" onclick={on_column_click}>
                {column_cells}
            </button>}
        })
        .collect::<Html>();

    board_html
}
