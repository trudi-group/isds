use super::*;

mod pow;
pub use pow::Pow;

#[function_component(Consensus)]
pub fn consensus() -> Html {
    html! {
        <StandardPage title="Consensus">
            <p class="block">
                { "This page might have more to tell you about consensus in the future." }
            </p>
            <p class="block">
                { "Right now, we only discuss: " }
                <Link<Route> to={Route::ConsensusPow}>
                    { "Proof-of-Work" }
                </Link<Route>>
                    { " (does it work?)" }
            </p>
        </StandardPage>
    }
}
