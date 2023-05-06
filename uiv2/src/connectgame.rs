use crate::cookies::get_player_id;
use crate::{database::get_object, Board, BoardView, Player};
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Properties)]
pub struct GameData {
    // TODO change usizes to fixed size (in Board too!)
    pub game_id: u64,
    pub board: Board,
    pub win_length: u8,
    pub turn_player: Player,
    pub win_status: Option<Player>,
    pub winning_chips: Option<HashSet<(usize, usize)>>,
    pub player1_id: u64,
    pub player2_id: u64,
}

impl GameData {
    pub fn new(
        width: u8,
        height: u8,
        win_length: u8,
        game_id: u64,
        player1_id: u64,
        player2_id: u64,
    ) -> Self {
        Self {
            game_id,
            board: Board::new(width, height),
            win_length,
            turn_player: Player::One,
            win_status: None,
            winning_chips: None,
            player1_id,
            player2_id,
        }
    }

    fn turn_player_id(&self) -> u64 {
        match self.turn_player {
            Player::One => self.player1_id,
            Player::Two => self.player2_id,
        }
    }

    // pub fn reset(&mut self) {
    //     let _ = mem::replace(
    //         self,
    //         Self::new(self.board.width, self.board.height, self.win_length),
    //     );
    // }
    // pub fn replace(&mut self, replacement: Self) {
    //     let _ = mem::replace(self, replacement);
    // }

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

#[derive(PartialEq)]
pub enum FetchGameData {
    NotFetching,
    Fetching,
    Success(GameData),
    Failed,
    InvalidId,
}

pub enum ConnectMsg {
    ColumnClick(usize),
    SetFetchState(FetchGameData),
    Reset,
    GetData,
}

#[derive(PartialEq, Properties)]
pub struct ConnectProps {
    pub game_id: u64,
}

pub struct ConnectGame {
    fetch_game_data: FetchGameData,
    game_data_cache: GameData,
}

impl ConnectGame {
    fn reset(&mut self) {
        // TODO Temp solution, might remove
        self.fetch_game_data = FetchGameData::NotFetching;
        self.game_data_cache = GameData::new(
            7,
            6,
            4,
            self.game_data_cache.game_id,
            self.game_data_cache.player1_id,
            self.game_data_cache.player2_id,
        );
    }
}

impl Component for ConnectGame {
    type Message = ConnectMsg;
    type Properties = ConnectProps; // maybe win_length should be in here to properly pass to board?

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            fetch_game_data: FetchGameData::NotFetching,
            game_data_cache: GameData::new(
                7,
                6,
                4,
                ctx.props().game_id,
                0, //TODO this is not ideal ofc
                0,
            ),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.fetch_game_data == FetchGameData::InvalidId {
            return html! {<h2>{"This game does not exist"}</h2>};
        }

        if self.fetch_game_data == FetchGameData::NotFetching {
            ctx.link().send_message(ConnectMsg::GetData);
        }

        let game_data = match &self.fetch_game_data {
            FetchGameData::Success(data) => data,
            _ => &self.game_data_cache,
        };

        let turn_player_html = match game_data.turn_player {
            Player::One => html! {<div class="smallblock">{"Player 1's turn"}</div>},
            Player::Two => html! {<div class="smallblock">{"Player 2's turn"}</div>},
        };

        let status_html = match game_data.win_status {
            None => turn_player_html,
            Some(Player::One) => {
                html! {<div class="smallblock red">{"Player 1 won!"}</div>}
            }
            Some(Player::Two) => {
                html! {<div class="smallblock blue">{"Player 2 won!"}</div>}
            }
        };

        let reset_click = ctx.link().callback(|_| ConnectMsg::Reset);
        let column_callbacks = (0..game_data.board.width)
            .map(|colnr| {
                ctx.link().callback(move |_| {
                    log::info!("Triggered column {}", colnr);
                    ConnectMsg::ColumnClick(colnr as usize)
                })
            })
            .collect::<Vec<_>>();
        html! { <>
            // <rect class="frame"/>

            {status_html}
            <div class="frame">
            <div class="grid">

            <BoardView board={game_data.board.clone()} winning_chips={game_data.winning_chips.clone()} column_callbacks={column_callbacks}/>
            // TODO: cloning isn't optimal. Possible solution: make board and winning_chips fields Rc<_> to allow sharing a reference
            // to the props
            </div>
            </div>
            <button onclick={reset_click} class="smallblock">{"Reset"}</button>
            // <DumbGet />
            </>
        }
    }

    // #[cfg(target_arch = "wasm32")]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // let id_state = use_state(|| 0u64);
        // let id_storage = Rc::new(RefCell::new(0));
        // let id_storage_clone = id_storage.clone();
        // spawn_local(async move {
        //     // let id_storage = &id_storage.clone();
        //     let id_string = Request::get("/api/getid")
        //         .send()
        //         .await
        //         .unwrap()
        //         .text()
        //         .await
        //         .unwrap();
        //     let id: u64 = id_string.parse().unwrap();
        //     id_storage_clone.replace(id);
        // });
        // let id = *id_storage.borrow();
        // log::info!("The id is {}", id);

        match msg {
            ConnectMsg::SetFetchState(state) => {
                self.fetch_game_data = state;
            }

            ConnectMsg::ColumnClick(colnr) => {
                let id = get_player_id();

                let mut game_data = match &self.fetch_game_data {
                    FetchGameData::Success(data) => data.clone(),
                    _ => return false, // TODO might have to change to game_data_cache. If so, make sure that any event impacting the
                                       // game state can only be done when fetch is successful
                                       // i.e. only turn_player can alter the DB
                };

                if game_data.win_status.is_some() {
                    return false;
                }

                if game_data.turn_player_id() != id {
                    // could check server side as well
                    log::info!(
                        "Ids don't match: 
                    game_data: {}
                    wasm_cookies: {}",
                        game_data.turn_player_id(),
                        id
                    );
                    return false;
                }

                let insert = game_data
                    .board
                    .insert(colnr, &game_data.turn_player.clone());
                // TODO: above we need to clone since we borrow self mutably for insert. Nice way without clone?
                match insert {
                    Ok(row) => {
                        if game_data.board.check_win(
                            colnr,
                            row,
                            &game_data.turn_player,
                            game_data.win_length as usize,
                        ) {
                            game_data.win_status = Some(game_data.turn_player.clone());
                            game_data.winning_chips = Some(game_data.board.find_winning_chips(
                                colnr,
                                row,
                                &game_data.turn_player,
                                game_data.win_length as usize,
                            ));
                            // log::info!("{:?}", self.winning_chips)
                        }
                        game_data.next_turn();
                        spawn_local(async move {
                            Request::post("/api/save_game")
                                .body(serde_json::to_string(&game_data).unwrap())
                                .send()
                                .await
                                .unwrap();
                        });
                        ctx.link()
                            .send_message(ConnectMsg::SetFetchState(FetchGameData::NotFetching));
                        // make view method fetch again
                    }
                    Err(_) => (), // do not switch turn, invalid move
                }
            }
            ConnectMsg::Reset => {
                let new_game_data = GameData::new(
                    7,
                    6,
                    4,
                    self.game_data_cache.game_id,
                    self.game_data_cache.player1_id,
                    self.game_data_cache.player2_id,
                );
                self.game_data_cache = new_game_data.clone();

                spawn_local(async move {
                    Request::post("/api/save_game")
                        .body(serde_json::to_string(&new_game_data).unwrap())
                        .send()
                        .await
                        .unwrap();
                });

                ctx.link()
                    .send_message(ConnectMsg::SetFetchState(FetchGameData::NotFetching));
            }
            ConnectMsg::GetData => {
                use ConnectMsg::SetFetchState;
                ctx.link()
                    .send_message(SetFetchState(FetchGameData::Fetching));

                let game_id = ctx.props().game_id.clone();
                ctx.link().send_future(async move {
                    match get_object(&format!("/api/gamedata/{}", game_id)).await {
                        //TODO maybe weird to get game id from props instead of GameData, but it is easiest
                        Ok(gamedata_opt) => match gamedata_opt {
                            Some(gamedata) => SetFetchState(FetchGameData::Success(gamedata)),
                            None => SetFetchState(FetchGameData::InvalidId),
                        },
                        Err(_) => SetFetchState(FetchGameData::Failed),
                    }
                });
            }
        }
        true
    }
}

#[function_component(UseState)]
fn state() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        Callback::from(move |_| counter.set(*counter + 1))
    };

    html! {
        <div>
            <button {onclick}>{ "Increment value" }</button>
            <p>
                <b>{ "Current value: " }</b>
                { *counter }
            </p>
        </div>
    }
}
