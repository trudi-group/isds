use super::*;

#[function_component(Hashes)]
pub fn hashes() -> Html {
    let (sim, node) = init_simulation();
    html! {
        <SimplePage title="What makes the blockchain so immutable?" footer=true >
            <div class="block pb-2">
                { "It's" }
                <span class="is-size-5 has-text-weight-bold mx-1">
                    { "hash functions." }
                </span>
            </div>
            <div class="block bp-2">
                <isds::Isds sim={ sim.into_shared() }>
                    <isds::BlockchainView
                        viewing_node={ node }
                        max_visible_blocks=8
                        show_unconfirmed_txes=false
                        show_header=false
                    />
                </isds::Isds>
            </div>
            <div class="block">
                {
                    indoc_markdown_content! { r#"
                        Every arrow ← in the blockchain above represents the result of invoking a *hash function*.
                        Blocks don't just contain transactions;
                        each block also contains in itself the *hash value* of the previous block.
                        This hash value, or just *hash* for short,
                        is the result of applying a hash function to all of the data in that block,
                        including the hash of the block before it.

                        Let's have a look at what hash functions do!
                        Below you can play around with `SHA-256` - the specific hash function used by Bitcoin.
                        `SHA-256` is a *cryptographic hash function*.
                        Among other things, this means that it's basically impossible
                        (with today's computers)
                        to find two inputs that have the same hash.
                        But feel free to try...
                        "#
                    }
                }
            </div>
            <div class="block">
                <div class="columns is-desktop is-centered is-vcentered">
                    <div class="column">
                        <div class="is-flex is-justify-content-center">
                            <div class="is-tight">
                                <isds::HashBox />
                            </div>
                        </div>
                    </div>
                    <div class="column">
                        {
                            indoc_markdown_content! { r#"
                                How does this work exactly? Well this is a question for a cryptography course...
                                The important thing to remember is that cryptographic hash functions help us
                                maintain the *integrity* of data.
                                If we keep the hash of a bunch of data in a safe place,
                                we can be pretty much certain that no one can modify the data,
                                not even a tiny bit,
                                without us noticing afterwards.
                                We only have to hash the data again and compare the result with the hash
                                we kept safe.

                                By trusting that the hash of a given block is correct\*
                                we can always check that all blocks up to this block
                                are in exactly the same state as they originally were.
                                Because whenever we change something,
                                for example the value of a transaction included in a given block,
                                we also change the input to our hash function,
                                and hence - with near certainty - also the resulting hash.
                                The blockchain is, in its essence, a *hash chain*.

                                Blocks for which the hashes don't match simply get rejected.
                                We either have no blocks, or we have blocks that weren't changed.
                                And therefore one could say that the blockchain is *immutable*\*\*.

                                \* *This* trust is earned at the [consensus layer](consensus).

                                \** We can still switch to a different chain of course,
                                respectively to a different *fork* of the blockchain.
                                This can also happen by accident, if something on the
                                [consensus](consensus) or [network](network) layers goes wrong.
                                The security of [each layer](/) in our model always depends in some way
                                on the security of the layers below.
                                "#
                            }
                        }
                    </div>
                </div>
            </div>
            <div class="block pt-5">
                <h3 class="title is-4">{"A note on cryptography..."}</h3>
                    {
                        indoc_markdown_content! { r#"
                                Cryptography is hard.
                                It's very easy to build something that you think is secure, but which just isn't.
                                Don't roll your own crypto\* - use what is crafted by cryptography experts
                                and battle-tested in practice!
                                And if some project you like does invent it's own cryptography -
                                a healthy dose of scepticism won't hurt...

                                \* The original meaning of "crypto" if of course "cryptography". What did *you* think?
                            "#
                        }
                    }
            </div>
        </SimplePage>
    }
}

fn init_simulation() -> (isds::Simulation, isds::Entity) {
    let mut sim = isds::Simulation::new();
    sim.add_event_handler(isds::InvokeProtocolForAllNodes(
        isds::nakamoto_consensus::NakamotoConsensus::default(),
    ));
    let node = sim.spawn_random_node();
    sim.do_now(isds::MultipleTimes::new(
        isds::ForSpecific(node, isds::nakamoto_consensus::MineBlock),
        20,
    ));
    sim.work_until(isds::SimSeconds::from(1.));
    sim.time.toggle_paused();
    (sim, node)
}
