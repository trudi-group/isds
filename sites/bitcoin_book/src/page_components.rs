use super::*;

#[derive(Properties, PartialEq)]
pub struct StandardPageProps {
    pub title: String,
    #[prop_or_default]
    pub children: Children,
}
#[function_component(StandardPage)]
pub fn standard_page(props: &StandardPageProps) -> Html {
    html! {
        <>
            <Header title={ props.title.clone() } />
            <Main>
                { for props.children.iter() }
            </Main>
        </>
    }
}

#[function_component(NavBar)]
pub fn nav_bar() -> Html {
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

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub title: String,
    #[prop_or_default]
    pub children: Children,
}
#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    html! {
        <header class="section">
            <div class="container">
                <NavBar />
                <h2 class="title">{ &props.title }</h2>
                { for props.children.iter() }
            </div>
        </header>
    }
}

#[derive(Properties, PartialEq)]
pub struct MainProps {
    #[prop_or_default]
    pub children: Children,
}
#[function_component(Main)]
pub fn main(props: &MainProps) -> Html {
    html! {
        <main class="section">
            <div class="container">
                { for props.children.iter() }
            </div>
        </main>
    }
}

#[function_component(Footer)]
pub fn footer() -> Html {
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
