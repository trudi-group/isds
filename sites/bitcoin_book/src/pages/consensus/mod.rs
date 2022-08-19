use super::*;

mod pow;
pub use pow::Pow;

// TODO a bit more text; and as markdown?

#[function_component(Consensus)]
pub fn consensus() -> Html {
    html! {
        <StandardPage title="Consensus">
            <div class="block">
                { "In the future, this page might have more to tell you about different consensus protocols." }
            </div>
            <div class="block">
                { "Right now, we only discuss " }
                <Link<Route> to={Route::ConsensusPow}>
                    { "Proof-of-Work" }
                </Link<Route>>
                    { " - a central element behind the Bitcoin network's approach to reaching consensus." }
            </div>
        </StandardPage>
    }
}
