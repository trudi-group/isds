use super::*;

#[derive(Properties, PartialEq)]
pub struct SimplePageProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub footer: bool,
    #[prop_or_default]
    pub children: Children,
}
/// Page with one section
#[function_component(SimplePage)]
pub fn simple_page(props: &SimplePageProps) -> Html {
    html! {
        <>
            <div class="section">
                <div class="container">
                    <NavBar />
                    <h2 class="title">{ &props.title }</h2>
                    { for props.children.iter() }
                </div>
            </div>
            if props.footer {
                <Footer />
            }
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
    pub title: AttrValue,
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
pub struct SectionProps {
    #[prop_or_default]
    pub children: Children,
}
#[function_component(Section)]
pub fn main(props: &SectionProps) -> Html {
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
