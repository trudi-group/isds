use super::*;

mod hashes;
pub use hashes::Hashes;

#[function_component(Blockchain)]
pub fn blockchain() -> Html {
    html! {
        <StandardPage title="The (actual) blockchain">
            <p class="block">
                { "This page might have more to tell you about blockchain-the-data-structure in the future." }
            </p>
            <p class="block">
                { "Right now, we only discuss: " }
                <Link<Route> to={Route::BlockchainHashes}>
                    { "What makes the blockchain so immutable?" }
                </Link<Route>>
            </p>
        </StandardPage>
    }
}
