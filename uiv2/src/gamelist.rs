use serde::{Deserialize, Serialize};
use yew_router::prelude::use_navigator;
// use surf;
use crate::cookies::get_player_id;
use crate::IdType;
use crate::{database::get_object, Pages};
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(PartialEq, Properties, Clone, Serialize, Deserialize, Debug)]
pub struct GameLobby {
    pub game_id: IdType,
    pub player1_id: Option<IdType>,
    pub player2_id: Option<IdType>,
    pub game_name: String, // TODO: yew recommends using their AttrValue instead
    // password: String
    pub game_started: bool,
}

#[derive(PartialEq, Clone, Copy)]
pub enum LobbyMode {
    Join,
    Open,
}

#[derive(PartialEq, Properties)]
pub struct GameLobbyBlockProps {
    gamelobby: GameLobby,
    mode: LobbyMode,
}

fn option_to_binary<T>(x: Option<T>) -> i32 {
    match x {
        Some(_) => 1,
        None => 0,
    }
}

impl GameLobby {
    pub fn number_players_joined(&self) -> i32 {
        option_to_binary(self.player1_id) + option_to_binary(self.player2_id)
    }
}

#[function_component(GameLobbyBlock)]
fn game_lobby_block(props: &GameLobbyBlockProps) -> Html {
    use LobbyMode::*;
    let game_id = props.gamelobby.game_id;
    let navigator = use_navigator().unwrap();
    let mode = props.mode;
    let onclick = Callback::from(move |_| {
        let navigator = navigator.clone();
        spawn_local(async move {
            if mode == Join {
                Request::get(&format!("/api/join/{}", game_id))
                    .send()
                    .await
                    .unwrap();
            }
            navigator.push(&Pages::Lobby {
                game_id: game_id.to_string(),
            });
        });
    });
    html! {
        <div class="gamelobby"> // TODO add class
            {&props.gamelobby.game_name}
            {format!("\n{}/2", props.gamelobby.number_players_joined())}
            // <form action="/api/join" method="post">
            //     <input type="hidden" name="game_id" value={gamelobby.game_id.to_string()}/>
            //     <input class="join" type="submit" value="Submit"/>
            // </form>
            <button class="greenbutton" onclick={onclick}> {match mode {
                Join => "Join!",
                Open => "Open"
            }} </button>
        </div>
    }
}

#[derive(PartialEq, Properties, Serialize, Deserialize, Debug)]
pub struct GameList {
    pub games: Vec<GameLobby>,
}

// #[function_component(GameListView)]
// pub fn game_list_view(gamelist: &GameList) -> Html {
// gamelist
//     .games
//     .iter()
//     .map(|game| html! {<GameLobbyBlock ..game.clone()/>})
//     .collect::<Html>()
// }

#[derive(PartialEq)]
pub enum FetchGameList {
    NotFetching,
    Fetching,
    Success((GameList, GameList)),
    Failed,
}

// #[derive(PartialEq)]
pub enum GameListMsg {
    SetFetchState(FetchGameList),
    GetData,
}

#[derive(PartialEq, Properties)]
pub struct GameListView {
    fetch_state: FetchGameList,
}

impl Component for GameListView {
    type Message = GameListMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            fetch_state: FetchGameList::NotFetching,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        use FetchGameList::*;
        if self.fetch_state == NotFetching {
            ctx.link().send_message(GameListMsg::GetData);
        }

        match &self.fetch_state {
            NotFetching => html! {"Please be patient"},
            Fetching => html! {"fetching open games"},
            Success((joined_gamelist, joinable_gamelist)) => html! {
            <>
            if joined_gamelist.games.len() > 0 {
                <h2>{"Continue playing"}</h2>
                {joined_gamelist
                .games
                .iter()
                .map(|game| html! {<GameLobbyBlock gamelobby={game.clone()} mode={LobbyMode::Open}/>})
                .collect::<Html>()}
            }

            <h2>{"Join a game"}</h2>
            if joinable_gamelist.games.len() > 0 {
                {joinable_gamelist
                    .games
                    .iter()
                    .map(|game| html! {<GameLobbyBlock gamelobby={game.clone()} mode={LobbyMode::Join}/>})
                    .collect::<Html>()}}
            else {
                <p> {"There aren't any open games. Go to the homepage to create a new one!"} </p>
            }
            </>},
            Failed => html! {"Failed to get the data. Please refresh to try again"},
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: GameListMsg) -> bool {
        use GameListMsg::*;
        match msg {
            SetFetchState(state) => {
                self.fetch_state = state;
                true
            }
            GetData => {
                ctx.link()
                    .send_message(SetFetchState(FetchGameList::Fetching));
                ctx.link().send_future(async {
                    // surf implementation
                    // match surf::get("/gamelistdata").recv_json::<GameList>().await {
                    //     Ok(gamelist) => return SetFetchState(FetchGameList::Success(gamelist)),
                    //     Err(_) => return SetFetchState(FetchGameList::Failed),
                    // };

                    // let gamelist_json = reqwasm::http::Request::get("/gamelistdata")
                    //     .send()
                    //     .await
                    //     .unwrap()
                    //     .text()
                    //     .await
                    //     .unwrap();

                    // let gamelist = serde_json::from_str(&gamelist_json).unwrap();
                    let player_id = get_player_id();
                    let joined_gamelist: GameList =
                        match get_object(&format!("/api/get_joined_lobbies/{}", player_id)).await {
                            Ok(gamelist) => gamelist,
                            Err(_) => return SetFetchState(FetchGameList::Failed),
                        };

                    let joinable_gamelist =
                        match get_object(&format!("/api/get_joinable_lobbies/{}", player_id)).await
                        {
                            Ok(gamelist) => gamelist,
                            Err(_) => return SetFetchState(FetchGameList::Failed),
                        };
                    SetFetchState(FetchGameList::Success((joined_gamelist, joinable_gamelist)))
                });
                true
            }
        }
    }
}
