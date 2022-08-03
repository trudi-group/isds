use super::*;

mod hashes;
pub use hashes::Hashes;

#[function_component(Blockchain)]
pub fn blockchain() -> Html {
    html! {
        <StandardPage title="The (actual) blockchain">
            <p>
                { "This page might have more to tell you about blockchain-the-data-structure in the future." }
            </p>
            <p>
                { "Right now, we only thematize: " }
                <Link<Route> to={Route::Hashes}>
                    { "What makes the blockchain so immutable?" }
                </Link<Route>>
            </p>
        </StandardPage>
    }
}
