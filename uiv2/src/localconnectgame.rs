use crate::{Board, BoardView, Player};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::mem;
use yew::prelude::*;

pub enum Msg {
    ColumnClick(usize),
    Reset,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct LocalGame {
    pub board: Board,
    pub win_length: usize,
    pub turn_player: Player,
    pub win_status: Option<Player>,
    pub winning_chips: Option<HashSet<(usize, usize)>>,
}

impl LocalGame {
    pub fn new(width: u8, height: u8, win_length: usize) -> Self {
        Self {
            board: Board::new(width, height),
            win_length,
            turn_player: Player::One,
            win_status: None,
            winning_chips: None,
        }
    }

    pub fn reset(&mut self) {
        let _ = mem::replace(
            self,
            Self::new(self.board.width, self.board.height, self.win_length),
        );
    }
    pub fn replace(&mut self, replacement: Self) {
        let _ = mem::replace(self, replacement);
    }

    // pub fn check_win(&self, col: usize, row: usize, player: &Player) -> Option<(i32, i32)> {
    //     self.board.check_win(col, row, player, self.win_length)
    // }

    pub fn next_turn(&mut self) {
        self.turn_player = match self.turn_player {
            Player::One => Player::Two,
            Player::Two => Player::One,
        }
    }
}

impl Component for LocalGame {
    type Message = Msg;
    type Properties = (); // maybe win_length should be in here to properly pass to board?

    fn create(_ctx: &Context<Self>) -> Self {
        Self::new(7, 6, 4)
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let turn_player_html = match self.turn_player {
            Player::One => html! {<div class="smallblock">{"Player 1's turn"}</div>},
            Player::Two => html! {<div class="smallblock">{"Player 2's turn"}</div>},
        };

        let status_html = match self.win_status {
            None => turn_player_html,
            Some(Player::One) => {
                html! {<div class="smallblock red">{"Player 1 won!"}</div>}
            }
            Some(Player::Two) => {
                html! {<div class="smallblock blue">{"Player 2 won!"}</div>}
            }
        };

        let reset_click = ctx.link().callback(|_| Msg::Reset);
        let column_callbacks = (0..self.board.width)
            .map(|colnr| {
                ctx.link()
                    .callback(move |_| Msg::ColumnClick(colnr as usize))
            })
            .collect::<Vec<_>>();
        html! { <>
            // <rect class="frame"/>

            {status_html}
            <div class="frame">
            <div class="grid">

            <BoardView board={self.board.clone()} winning_chips={self.winning_chips.clone()} column_callbacks={column_callbacks}/>
            // TODO: cloning isn't optimal. Possible solution: make board and winning_chips fields Rc<_> to allow sharing a reference
            // to the props
            </div>
            </div>
            <button onclick={reset_click} class="smallblock">{"Reset"}</button>
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ColumnClick(colnr) => {
                if self.win_status.is_some() {
                    return false;
                }
                let insert = self.board.insert(colnr, &self.turn_player.clone());
                // TODO: above we need to clone since we borrow self mutably for insert. Nice way without clone?
                match insert {
                    Ok(row) => {
                        if self
                            .board
                            .check_win(colnr, row, &self.turn_player, self.win_length)
                        {
                            self.win_status = Some(self.turn_player.clone());
                            self.winning_chips = Some(self.board.find_winning_chips(
                                colnr,
                                row,
                                &self.turn_player,
                                self.win_length,
                            ));
                            // log::info!("{:?}", self.winning_chips)
                        }
                        self.next_turn()
                    }
                    Err(_) => (), // do not switch turn, invalid move
                }
            }
            Msg::Reset => {
                self.reset();
            }
        }
        true
    }
}
