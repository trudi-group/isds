use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

#[macro_use]
mod markdown;

mod keyboard;
use keyboard::{
    init_keyboard_listener, init_keyboard_listener_with_block_size_limit, KeyboardShortcutsList,
};

mod user_model;
use user_model::{random_transaction, random_transaction_from_random_node};

mod page_components;
use page_components::{Footer, Header, Section, SimplePage};

mod pages;

#[derive(Copy, Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Layers,
    #[at("/application")]
    ToDo,
    #[at("/blockchain")]
    Blockchain,
    #[at("/blockchain/hashes")]
    BlockchainHashes,
    #[at("/consensus")]
    Consensus,
    #[at("/consensus/pow")]
    ConsensusPow,
    #[at("/network")]
    Network,
    #[at("/network/standalone")]
    NetworkStandalone,
    #[at("/beyond")]
    Beyond,
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl Route {
    fn resolve(&self) -> Html {
        match self {
            Route::Layers => html! { <pages::Layers /> },
            Route::Blockchain => html! { <pages::Blockchain /> },
            Route::BlockchainHashes => html! { <pages::blockchain::Hashes /> },
            Route::Consensus => html! { <pages::Consensus /> },
            Route::ConsensusPow => html! { <pages::consensus::Pow /> },
            Route::Network => html! { <pages::Network /> },
            Route::NetworkStandalone => html! { <pages::network::Standalone /> },
            Route::Beyond => html! { <pages::Beyond /> },
            Route::ToDo => html! {
                <SimplePage title="TODO">
                    { include_markdown_content!("pages/todo.md") }
                </SimplePage>
            },
            Route::NotFound => html! {
                <SimplePage title="Not Found">
                    { include_markdown_content!("pages/404.md") }
                </SimplePage>
            },
        }
    }
    fn nav_name(&self) -> &str {
        match self {
            Route::Layers => "Layers",
            Route::Blockchain => "Blockchain",
            Route::BlockchainHashes => "Hashes",
            Route::Consensus => "Consensus",
            Route::ConsensusPow => "Proof of Work (PoW)",
            Route::Network => "Network",
            Route::NetworkStandalone => "Standalone",
            Route::Beyond => "Beyond",
            Route::ToDo => "TODO",
            Route::NotFound => "404",
        }
    }
    fn parent(&self) -> Option<Self> {
        match self {
            Route::BlockchainHashes => Some(Route::Blockchain),
            Route::ConsensusPow => Some(Route::Consensus),
            Route::NetworkStandalone => Some(Route::Network),
            Route::Layers => None,
            _ => Some(Route::Layers),
        }
    }
}

#[function_component(BitcoinBook)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render = {Switch::render(Route::resolve)} />
        </BrowserRouter>
    }
}

fn main() {
    yew::start_app::<BitcoinBook>();
}
