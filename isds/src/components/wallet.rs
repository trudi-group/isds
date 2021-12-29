use super::*;

pub struct Wallet {
    sim: SharedSimulation,
    _context_handle: yew::context::ContextHandle<IsdsContext>,
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Rendered(RealSeconds),
}

impl Component for Wallet {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (context_data, _context_handle) = get_isds_context!(ctx, Self);

        let sim = context_data.sim;
        Self {
            sim,
            _context_handle,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <div class="box">
                <div>
                    <span class="is-size-3">
                        { "0.0001 coins" }
                    </span>
                    <span class="ml-2 is-size-5">
                        { "(-0.3 pending)" }
                    </span>
                </div>
                <table class="table">
                    <tbody>
                        <tr>
                            <td>
                                <span class="icon is-size-6 has-text-warning">
                                    { "0/3" }
                                </span>
                            </td>
                            <td>
                                <span class="has-text-grey-light is-family-code">
                                    { "1Archive1n2C579dMsAu3iC6tWzuQJz8dN" } // achive.org donation address
                                </span>
                            </td>
                            <td>
                                <span class="has-text-warning has-text-weight-medium">
                                    { "-0.3" }
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td>
                                <span class="icon has-text-danger">
                                    <i class="fas fa-circle"></i>
                                </span>
                            </td>
                            <td>
                                <span class="has-text-grey is-family-code">
                                    { "1Archive1n2C579dMsAu3iC6tWzuQJz8dN" } // achive.org donation address
                                </span>
                            </td>
                            <td>
                                <span class="has-text-danger has-text-weight-medium">
                                    { "-0.3" }
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td>
                                <span class="icon has-text-success">
                                    <i class="fas fa-circle"></i>
                                </span>
                            </td>
                            <td>
                                <span class="has-text-grey is-family-code">
                                    { "Alice" }
                                </span>
                            </td>
                            <td>
                                <span class="has-text-success has-text-weight-medium">
                                    { "+2.3" }
                                </span>
                            </td>
                        </tr>
                    </tbody>
                </table>
                <div class="buttons">
                    <button class="button">
                        <span class="icon">
                            <i class="fas fa-paper-plane fa-rotate-90"></i>
                        </span>
                        <span>
                            { "Request coins" }
                        </span>
                    </button>
                    <button class="button">
                        <span>
                            { "Send coins" }
                        </span>
                        <span class="icon">
                            <i class="fas fa-paper-plane"></i>
                        </span>
                    </button>
                </div>
            </div>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Rendered(_) => {
                true // TODO?
            }
        }
    }
}
