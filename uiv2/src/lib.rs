// TODO: remove common, replace ui stuff. It's too much hassle to have to create a wrapper class to impl
// yew stuff
use core::fmt;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;

pub mod board;
mod cell;
pub mod gamelist;
use board::{Board, BoardView};
use gamelist::GameListView;
mod database;
mod homepage;
use homepage::HomePage;
pub mod connectgame;
mod lobby;
use lobby::GameLobbyView;
mod localconnectgame;
use connectgame::ConnectGame;
use localconnectgame::LocalGame;
mod notfound;
use notfound::NotFoundPage;
pub mod cookies;

pub type IdType = u32;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]

pub enum Player {
    One,
    Two,
}

impl Into<u8> for Player {
    fn into(self) -> u8 {
        match self {
            Player::One => 1,
            Player::Two => 2,
        }
    }
}

impl TryFrom<u8> for Player {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Player::One),
            2 => Ok(Player::Two),
            _ => Err("Invalid player identifier".to_owned()),
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "p{}",
            match self {
                Player::One => "1",
                Player::Two => "2",
            }
        )
    }
}

#[derive(PartialEq, Clone, Routable)]
pub enum Pages {
    #[at("/")]
    HomePage,
    #[at("/game/:game_id")]
    Game { game_id: IdType },
    #[at("/gamelist")]
    GameList,
    #[at("/lobby/:game_id")]
    Lobby { game_id: String },
    #[at("/localgame")]
    Local,
    #[not_found]
    #[at("/404")]
    NotFound,
}

// #[derive(PartialEq, Properties, Default)]
// pub struct AppProps {
//     page: Pages,
// }

// #[function_component]
// pub fn App(props: &AppProps) -> Html {
//     match props.page {
//         Pages::HomePage => html! {<HomePage/>},
//         Pages::GameList => html! { <GameListView/>},
//         Pages::ConnectGame => html! {<GameData/>},
//     }
// }

fn switch(routes: Pages) -> Html {
    match routes {
        Pages::HomePage => html! {<HomePage/>},
        Pages::GameList => html! { <GameListView/>},
        Pages::Game { game_id } => html! {<ConnectGame game_id={game_id}/>},
        Pages::Lobby { game_id } => {
            html! {<GameLobbyView game_id = {game_id.parse::<IdType>().unwrap()}/>}
        }
        Pages::Local => html! {<LocalGame/>},
        Pages::NotFound => html! {<NotFoundPage/>},
    }
}

#[function_component]
fn Title() -> Html {
    let navigator = use_navigator().unwrap();
    let return_homepage = Callback::from(move |_| navigator.push(&Pages::HomePage));
    html! {<button onclick={return_homepage} style="all:unset;cursor:pointer;">
        <h1>{"Connect 4 on a Torus"}</h1>
    </button>}
}

// #[function_component]
// fn Counter() -> Html {
//     let count = use_state(|| 0);
//     let count_clone = count.clone();
//     let onclick = Callback::from(move |_| count.set(*count + 1));
//     html! {
//         <>
//         <button onclick={onclick}>{"Add 1"}</button>
//         <p>{*count_clone}</p>
//         </>
//     }
// }

#[function_component]
pub fn App() -> Html {
    wasm_logger::init(wasm_logger::Config::default());

    html! {
        <div class="mainpage">
        <BrowserRouter>
            <Title/>
            <Switch<Pages> render={switch} /> // <- must be child of <BrowserRouter>
        </BrowserRouter>
        </div>
    }
}

// struct ConnectGame {
//     gamedata: GameData,
//     player1_id: usize,
//     player2_id: usize,
//     game_id: usize,
// }

// #[allow(dead_code)]
// fn main() {
//     // Logging
//     // wasm_logger::init(wasm_logger::Config::default());
//     // yew::Renderer::<GameData>::new().render();
//     yew::Renderer::<GameData>::new().render();
//     // yew::start_app::<GameData>();
// }
