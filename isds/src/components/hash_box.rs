#![macro_use]
extern crate gloo;
use gloo::console::log;

use yew::prelude::*;

use web_sys::HtmlInputElement;

use sha2::digest::generic_array::{typenum::U32, GenericArray};
use sha2::{Digest, Sha256};

pub struct HashBox {
    input_ref: NodeRef,
}

pub enum Msg {
    InputChanged,
}

impl Component for HashBox {
    type Message = Msg;
    type Properties = ();

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|_| Msg::InputChanged);

        let input_value = self
            .input_ref
            .cast::<HtmlInputElement>()
            .map(|ie| ie.value())
            .unwrap_or_default();
        let hash_value = sha256(&input_value);

        html! {
            <div class="box">
                <div class="field">
                    <label class="label">{"Type anything:"}</label>
                    <input ref={self.input_ref.clone()} class="input is-size-7" {oninput} />
                </div>
                <div class="field">
                    <label class="label">{"The SHA256 hash of what you just typed, as a hex string:"}</label>
                    <input class="input is-size-7" readonly=true value={format!("{:x}", hash_value)} />
                </div>
                <div class="field">
                    <label class="label">{"Expressed as bits (32 bits per line):"}</label>
                    <table class="table">
                        <thead>
                            <tr>
                                <th>{"hex"}</th>
                                <th>{"binary"}</th>
                            </tr>
                        </thead>
                        <tbody class="is-family-code">
                            {(0..32).step_by(4).map(|i| {
                                html!{
                                    <tr>
                                        <td>{format!("{:02x} {:02x} {:02x} {:02x}", hash_value[i], hash_value[i+1], hash_value[i+2], hash_value[i+3])}</td>
                                        <td>{format!("{:08b} {:08b} {:08b} {:08b}", hash_value[i], hash_value[i+1], hash_value[i+2], hash_value[i+3])}</td>
                                    </tr>
                               }
                            }).collect::<Vec<Html>>()}
                        </tbody>
                    </table>
                </div>
            </div>
        }
    }
    fn create(_: &Context<Self>) -> Self {
        Self {
            input_ref: NodeRef::default(),
        }
    }
    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        log!("Got update!");
        true
    }
}

fn sha256(input: &str) -> GenericArray<u8, U32> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize()
}

// ------ ------
//     Tests
// ------ ------

#[cfg(test)]
mod wtests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn sha256_hashes_correctly() {
        let actual = format!("{:x}", sha256("Hello!"));
        let expected = "334d016f755cd6dc58c53a86e183882f8ec14f52fb05345887c8a5edd42c87b7";
        assert_eq!(expected, actual);
    }
}
