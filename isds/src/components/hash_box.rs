use super::*;

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
    pub existing_data: Option<AttrValue>,
    #[prop_or(false)]
    pub show_only_first_32_bits: bool,
    #[prop_or(true)]
    pub show_hex: bool,
    /// hides children while target is not reached
    #[prop_or(0)]
    pub leading_zero_bits_target: usize,
    #[prop_or(false)]
    pub block_on_reached_target: bool,
    #[prop_or(false)]
    pub highlight_leading_zero_bits: bool,
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
            show_only_first_32_bits,
            leading_zero_bits_target,
            highlight_leading_zero_bits,
            ..
        } = ctx.props();
        let existing_data = ctx.props().existing_data.as_ref();

        let user_input = self
            .input_ref
            .cast::<HtmlInputElement>()
            .map(|ie| ie.value())
            .unwrap_or_default();
        let input_value = if let Some(existing_data) = existing_data {
            format!("{existing_data}\n{user_input}")
        } else {
            user_input
        };
        let hash_value = sha256(&input_value);

        let last_shown_byte = if show_only_first_32_bits { 3 } else { 31 };
        let leading_zero_bits = leading_zero_bits(hash_value);
        let target_reached = leading_zero_bits >= leading_zero_bits_target;

        html! {
            <div class="box">
                if let Some(existing_data) = existing_data {
                    <div class="field">
                        <label class="label">{"Existing data:"}</label>
                        <input
                            class="input is-size-7 is-family-code"
                            readonly=true
                            value={ existing_data.clone() }
                        />
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
                            if existing_data.is_some() {
                                "The resulting SHA256 hash, as a hex string:"
                            } else {
                                "The SHA256 hash of what you just typed, as a hex string:"
                            }
                        }
                    </label>
                    <input
                        class="input is-size-7 is-family-code"
                        readonly=true
                        value={format!("{:x}", hash_value)}
                    />
                </div>
                <div class="field">
                    <label class="label">
                        if show_only_first_32_bits {
                            { "First 32 bits:" }
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
                                (0..=last_shown_byte).step_by(4).map(|i| {
                                    html!{
                                        <tr>
                                            if show_hex {
                                                <td>
                                                    { view_hex(hash_value, i) }
                                                </td>
                                            }
                                            <td>
                                                {
                                                    view_bits(
                                                        hash_value,
                                                        i,
                                                        highlight_leading_zero_bits)
                                                }
                                            </td>
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
    fn changed(&mut self, _: &Context<Self>) -> bool {
        if let Some(el) = self.input_ref.cast::<HtmlInputElement>() {
            el.set_value("")
        }
        true
    }
}

fn view_hex(hash: GenericArray<u8, U32>, i: usize) -> Html {
    html! {
        {
            format!("{:02x} {:02x} {:02x} {:02x}", hash[i], hash[i+1], hash[i+2], hash[i+3])
        }
    }
}

fn view_bits(hash: GenericArray<u8, U32>, i: usize, highlight_leading_zero_bits: bool) -> Html {
    let leading_zero_bits = leading_zero_bits(hash);
    html! {
        if highlight_leading_zero_bits && i < leading_zero_bits {
            {
                // whole highlighted bytes up to and excluding the last such byte
                (i..i+4).take_while(|j| (j + 1) * 8 < leading_zero_bits)
                    .map(|j| html! {
                        <>
                            <span class="has-text-weight-bold is-underlined">
                            { format!("{:08b}", hash[j]) }
                            </span>
                            { " " }
                        </>
                    }).collect::<Html>()
            }
            if leading_zero_bits > 0 && leading_zero_bits % 8 > 0 {
                // the partially-highlighted byte
                <span class="has-text-weight-bold is-underlined">
                {
                    ("0").repeat(leading_zero_bits % 8)
                }
                </span>
                {
                    format!("{:b}", hash[leading_zero_bits / 8])
                }
                { " " }
            } else {
                // the last byte with leading zeros is an all-zero byte
                <span class="has-text-weight-bold is-underlined">
                    { format!("{:08b}", hash[leading_zero_bits / 8 - 1]) }
                </span>
                { " " }
            }
            {
                // the remaining bytes
                (i..i+4).skip_while(|j| j * 8 < leading_zero_bits)
                    .map(|j| format!("{:08b} ", hash[j])).collect::<Html>()
            }
        } else {
            {format!("{:08b} {:08b} {:08b} {:08b}", hash[i], hash[i+1], hash[i+2], hash[i+3])}
        }
    }
}

fn sha256(input: &str) -> GenericArray<u8, U32> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize()
}

fn leading_zero_bits(hash: GenericArray<u8, U32>) -> usize {
    hash.iter().copied().take_while(|&byte| byte == 0).count() * 8
        + hash
            .iter()
            .copied()
            .skip_while(|&byte| byte == 0)
            .map(|byte| byte.leading_zeros() as usize)
            .next()
            .unwrap_or_default()
}

// ------ ------
//     Tests
// ------ ------

#[cfg(test)]
mod wtests {
    use super::*;
    use generic_array::arr;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn sha256_hashes_correctly() {
        let actual = format!("{:x}", sha256("Hello!"));
        let expected = "334d016f755cd6dc58c53a86e183882f8ec14f52fb05345887c8a5edd42c87b7";
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn leading_zeroes_counted_correctly_with_4_zeroes() {
        let input = arr![u8;
            8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let actual = leading_zero_bits(input);
        let expected = 4;
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn leading_zeroes_counted_correctly_with_9_zeroes() {
        let input = arr![u8;
            0, 64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
        let actual = leading_zero_bits(input);
        let expected = 9;
        assert_eq!(expected, actual);
    }
}
