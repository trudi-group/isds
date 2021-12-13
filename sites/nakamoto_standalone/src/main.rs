use yew::prelude::*; // TODO make this a reexport of isds maybe? check how yew example do this
use isds;

struct NakamotoStandalone;

impl Component for NakamotoStandalone {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html!{
            <isds::Isds>
                <isds::FpsCounter />
                <br />
                <isds::NetView />
            </isds::Isds>
        }
    }
}

fn main() {
    yew::start_app::<NakamotoStandalone>();
}
