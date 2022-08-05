use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

#[macro_use]
mod utils;

mod pages;
mod user;

#[derive(Copy, Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Layers,
    #[at("/blockchain")]
    Blockchain,
    #[at("/blockchain/hashes")]
    BlockchainHashes,
    #[at("/network")]
    Network,
    #[at("/network/standalone")]
    NetworkStandalone,
    #[at("/beyond")]
    Beyond,
    #[at("/todo")]
    ToDo,
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
            Route::Network => html! { <pages::Network /> },
            Route::NetworkStandalone => html! { <pages::network::Standalone /> },
            Route::Beyond => html! { <pages::Beyond /> },
            Route::ToDo => html! {
                <StandardPage title="TODO">
                    { include_markdown_content!("pages/todo.md") }
                </StandardPage>
            },
            Route::NotFound => html! {
                <StandardPage title="Not Found">
                    { include_markdown_content!("pages/404.md") }
                </StandardPage>
            },
        }
    }
    fn nav_name(&self) -> &str {
        match self {
            Route::Layers => "Layers",
            Route::Blockchain => "Blockchain",
            Route::BlockchainHashes => "Hashes",
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
            Route::Layers => None,
            _ => Some(Route::Layers),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct StandardPageProps {
    pub title: String,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(StandardPage)]
fn standard_page(props: &StandardPageProps) -> Html {
    html! {
        <>
            <header class="section">
                <NavBar />
                <h2 class="title">{ &props.title }</h2>
            </header>
            <main class="section">
                { for props.children.iter() }
            </main>
        </>
    }
}

#[function_component(NavBar)]
fn nav_bar() -> Html {
    let route: Route = use_route().unwrap_or_default();
    html! {
            <nav class="breadcrumb">
                <ul>
                    if let Some(grandparent) = route.parent().and_then(|route| route.parent()) {
                        <li>
                            <Link<Route> to={ grandparent }>
                                { grandparent.nav_name() }
                            </Link<Route>>
                        </li>
                    }
                    if let Some(parent) = route.parent() {
                        <li>
                            <Link<Route> to={ parent }>
                                { parent.nav_name() }
                            </Link<Route>>
                        </li>
                    }
                    <li class="is-active">
                        <Link<Route> to={route}>{ route.nav_name() }</Link<Route>>
                    </li>
                </ul>
            </nav>
    }
}

#[function_component(Footer)]
fn footer() -> Html {
    html! {
        <footer class="footer">
            <div class="container">
                { "Built with " }
                <i class="fas fa-heart"></i>
                { " at the " }
                <a href="https://www.weizenbaum-institut.de/en/research/rg17/">{ "Weizenbaum Institute" }</a>
                { ". Source code " }
                <a href="https://github.com/trudi-group/isds">{ "here" }</a>
                { ". Feedback " }
                <a href="mailto:martin.florian@hu-berlin.de">{ "welcome" }</a>
                { "!" }
            </div>
        </footer>
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
