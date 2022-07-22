#[derive(Debug, Clone)]
pub struct PseudorandomColors {
    full_palette: Vec<String>,
}

pub const DEFAULT_SEED_PALETTE: &[&str] = &[
    // WI colors
    "#A69D82", // greige
    "#7D505A", // mauve
    "#235A82", // blue
    "#46695A", // darkgreen
    "#829664", // lightgreen
    "#C88C28", // yellow
    "#BE552D", // orange
];

impl PseudorandomColors {
    pub fn new(seed_palette: &[&str], target_palette_n: usize) -> Self {
        use palette::{FromColor, Gradient, Lab, Pixel, Srgb};
        use std::str::FromStr;
        assert!(seed_palette.len() <= target_palette_n);

        let seed_colors = seed_palette.iter().map(|c| Srgb::from_str(c).unwrap());
        let gradient = Gradient::new(
            seed_colors.map(|c| Lab::from_color(c.into_format::<f32>().into_linear())),
        );

        let full_palette = gradient
            .take(target_palette_n)
            .map(|c| {
                format!(
                    "#{}",
                    hex::encode(Srgb::from_color(c).into_format().into_raw::<[u8; 3]>())
                )
            })
            .collect();

        Self { full_palette }
    }
    pub fn get(&self, number: u32) -> &str {
        let index = pseudorandomize(number) as usize % self.full_palette.len();
        &self.full_palette[index]
    }
    pub fn all(&self) -> &[String] {
        &self.full_palette
    }
}

fn pseudorandomize(number: u32) -> u32 {
    // inspired by legion's `U64Hasher`
    let big_prime = 2u32.pow(31) - 1; // eighth Mersenne prime, largest prime in `u32`
    big_prime.wrapping_mul(number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn parse_one_html_color() {
        let colors = PseudorandomColors::new(&["#008000"], 1);
        let expected = vec!["#008000"];
        let actual = colors.full_palette;
        assert_eq!(expected, actual);
    }

    #[wasm_bindgen_test]
    fn blend_between_multiple_html_colors() {
        let colors = PseudorandomColors::new(&["#008000", "#0000FF", "#ffff00"], 5);
        assert_eq!(colors.full_palette[0], "#008000");
        assert_ne!(colors.full_palette[1], "#008000");
        assert_ne!(colors.full_palette[1], "#0000ff");
        assert_eq!(colors.full_palette[2], "#0000ff");
        assert_ne!(colors.full_palette[3], "#0000ff");
        assert_ne!(colors.full_palette[3], "#ffff00");
        assert_eq!(colors.full_palette[4], "#ffff00");
    }

    #[wasm_bindgen_test]
    fn pseudorandom_is_random_but_deterministic() {
        let colors = PseudorandomColors::new(&["#008000", "#0000FF", "#ffff00"], 1024);
        assert_eq!(colors.get(42), colors.get(42));
        assert_ne!(colors.get(23), colors.get(42));
    }
}
