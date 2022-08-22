use super::*;

mod pow;
pub use pow::Pow;

#[function_component(Consensus)]
pub fn consensus() -> Html {
    html! {
        <SimplePage title="Consensus">
            <div class="block">
                { "In the future, this page might have more to tell you about different consensus protocols." }
            </div>
            <div class="block">
                { "Right now, we only discuss " }
                <Link<Route> to={Route::ConsensusPow}>
                    { "Proof-of-Work" }
                </Link<Route>>
                    { " - a central pillar of Bitcoin's approach to consensus." }
            </div>
        </SimplePage>
    }
}
