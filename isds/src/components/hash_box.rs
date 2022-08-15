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

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub seed: Option<String>,
    #[prop_or(false)]
    pub show_only_last_32_bits: bool,
    #[prop_or(true)]
    pub show_hex: bool,
    /// hides children while target is not reached
    #[prop_or(0)]
    pub zeroes_target: usize,
    #[prop_or(false)]
    pub block_on_reached_target: bool,
    #[prop_or_default]
    pub children: Children,
}

impl Component for HashBox {
    type Message = Msg;
    type Properties = Props;

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|_| Msg::InputChanged);

        let &Props {
            block_on_reached_target,
            show_hex,
            show_only_last_32_bits,
            zeroes_target,
            ..
        } = ctx.props();
        let seed = ctx.props().seed.as_ref();

        let user_input = self
            .input_ref
            .cast::<HtmlInputElement>()
            .map(|ie| ie.value())
            .unwrap_or_default();
        let input_value = if let Some(seed) = seed {
            format!("{seed}\n{user_input}")
        } else {
            user_input
        };
        let hash_value = sha256(&input_value);

        let first_shown_byte = if show_only_last_32_bits { 28 } else { 0 };
        let target_reached = trailing_zeroes(hash_value) >= zeroes_target;

        html! {
            <div class="box">
                if let Some(seed) = seed {
                    <div class="field">
                        <label class="label">{"Existing data:"}</label>
                        <input class="input is-size-7 is-family-code" readonly=true value={ seed.clone() } />
                    </div>
                }
                <div class="field">
                    <label class="label">{"Type anything:"}</label>
                    <input
                        ref={self.input_ref.clone()}
                        {oninput}
                        class="input is-size-7"
                        readonly={ block_on_reached_target && target_reached }
                    />
                </div>
                <div class="field">
                    <label class="label">
                        {
                            if seed.is_some() {
                                "The resulting SHA256 hash, as a hex string:"
                            } else {
                                "The SHA256 hash of what you just typed, as a hex string:"
                            }
                        }
                    </label>
                    <input class="input is-size-7 is-family-code" readonly=true value={format!("{:x}", hash_value)} />
                </div>
                <div class="field">
                    <label class="label">
                        if show_only_last_32_bits {
                            { "Last 32 bits:" }
                        } else {
                            { "Expressed as bits (32 bits per line):" }
                        }
                    </label>
                    <table class="table">
                        if show_hex {
                            <thead>
                                <tr>
                                    <th>{"hex"}</th>
                                    <th>{"binary"}</th>
                                </tr>
                            </thead>
                        }
                        <tbody class="is-family-code">
                            {
                                (first_shown_byte..32).step_by(4).map(|i| {
                                    html!{
                                        <tr>
                                            if show_hex {
                                                <td>{format!("{:02x} {:02x} {:02x} {:02x}", hash_value[i], hash_value[i+1], hash_value[i+2], hash_value[i+3])}</td>
                                            }
                                            <td>{format!("{:08b} {:08b} {:08b} {:08b}", hash_value[i], hash_value[i+1], hash_value[i+2], hash_value[i+3])}</td>
                                        </tr>
                                   }
                                }).collect::<Vec<Html>>()
                            }
                        </tbody>
                    </table>
                </div>
                if target_reached {
                    { for ctx.props().children.iter() }
                }
            </div>
        }
    }
    fn create(_: &Context<Self>) -> Self {
        Self {
            input_ref: NodeRef::default(),
        }
    }
    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        true
    }
}

fn sha256(input: &str) -> GenericArray<u8, U32> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize()
}

fn trailing_zeroes(hash: GenericArray<u8, U32>) -> usize {
    hash.iter().rev().take_while(|&&byte| byte == 0).count() * 8
        + hash
            .iter()
            .rev()
            .skip_while(|&&byte| byte == 0)
            .map(|&byte| {
                if byte == 0 {
                    8
                } else if byte % 128 == 0 {
                    7
                } else if byte % 64 == 0 {
                    6
                } else if byte % 32 == 0 {
                    5
                } else if byte % 16 == 0 {
                    4
                } else if byte % 8 == 0 {
                    3
                } else if byte % 4 == 0 {
                    2
                } else if byte % 2 == 0 {
                    1
                } else {
                    0
                }
            })
            .next()
            .unwrap_or_default()
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

    #[wasm_bindgen_test]
    fn trailing_zeroes_counted_correctly_with_4_zeroes() {
        let actual = trailing_zeroes(sha256("abcdefghij"));
        let expected = 4;
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn trailing_zeroes_counted_correctly_with_9_zeroes() {
        let actual = trailing_zeroes(sha256(
            "111111111111111111111111111111111111111111111111111111111111111111111111111111",
        ));
        let expected = 9;
        assert_eq!(expected, actual);
    }
}
