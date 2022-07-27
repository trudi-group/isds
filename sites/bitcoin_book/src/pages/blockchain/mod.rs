use super::*;

#[function_component(Blockchain)]
pub fn blockchain() -> Html {
    html! {
        <StandardPage title="The (actual) blockchain">
            <h3>
                { "What makes the blockchain so immutable?" }
            </h3>
            <p>
                { "Hash functions." }
            </p>
            <p>
                { "And this is how hash functions work:" }
            </p>
            <isds::HashBox />
            <p>
                { "This page is " }
                <Link<Route> to={Route::ToDo}>
                    { "work in progress " }
                </Link<Route>>
                { "by the way..." }
            </p>
        </StandardPage>
    }
}
