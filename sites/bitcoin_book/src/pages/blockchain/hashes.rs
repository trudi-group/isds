use super::*;

#[function_component(Hashes)]
pub fn hashes() -> Html {
    html! {
        <StandardPage title="What makes the blockchain so immutable?">
            <p class="block">
                { "It's hash functions." }
            </p>
            <p class="block">
                { "And this is what hash functions do:" }
            </p>
            <div class="block is-tight">
                <isds::HashBox />
            </div>
            <p class="block">
                {
                    indoc_markdown_content! { r#"
                        `SHA-256` that we showcase above is a *cryptographic hash function*.
                        Among other things, this means that it's basically impossible
                        (with today's computers)
                        to find two inputs that have the same hash.

                        If each block in our blockchain contains a cryptographic hash
                        of the data stored in the block before it
                        (and essentially they do),
                        then whenever someone changes something in *any* earlier block,
                        we will be able to detect it.
                        Do you see why?

                        *(insert image of a blockchain here)*

                        Being able to detect blocks that have been tampered with
                        means that we can reject and discard them,
                        such that our internal state will never be "wrong"...
                        Unless of course there is a problem on the
                        [consensus](todo) or [network](network) layers.
                        "#
                    }
                }
            </p>
        </StandardPage>
    }
}
