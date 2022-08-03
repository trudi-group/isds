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
            Route::Beyond => "Beyond",
            Route::ToDo => "TODO",
            Route::NotFound => "404",
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
                    if route != Route::Layers {
                        <li>
                            <Link<Route> to={Route::Layers}>
                                { Route::Layers.nav_name() }
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
