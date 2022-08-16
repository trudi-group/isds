use super::*;
pub use std::error::Error;

#[function_component(Pow)]
pub fn pow() -> Html {
    html! {
        <StandardPage title="Proof-of-Work (PoW)">
            <p class="block">
                {
                    indoc_markdown_content! { r#"
                        Bitcoin, as well as most other cryptocurrency networks,
                        is what is called a *permissionless* system.
                        This means that anyone can become an active part of the network and,
                        for example,
                        propose new blocks.
                        No participant needs to register anywhere or ask for permission.
                        Between the lines, this means that we don't really know who is
                        responsible for the messages that get sent in the network.
                        This also implies that it's hard to do something like *voting*.
                        We have a problem if we want to decide democratically what should be the correct next block,
                        for example.
                        There is no way to enforce that each participant gets one vote,
                        or even a limited number of votes.

                        In the example below, the top nodes are competitors,
                        each tries to convince the third node that *its* chain is the right one.
                        The example uses the *longest chain rule* -
                        the bottom node will assume that the longest chain it knows is the right one.
                        However, creating blocks is really easy...
                        Try it!
                        Then try confusing the bottom node!
                        "#
                    }
                }
            </p>

            <div class="block">
                <NoPowExample />
            </div>

            <p class="block">
                {
                    indoc_markdown_content! { r#"
                        This is where Proof-of-Work (PoW) comes in.
                        PoW means that a node needs to prove that she put
                        hard work into that last block.
                        The underlying assumption here is that the ability to do work is more or
                        less fairly distributed.

                        What is work?
                        Well, unfortunately it can't be anything that is intrinsically valuable...
                        So essentially, work in the context of Bitcoin's PoW is *solving puzzles*.

                        What kind of puzzles?
                        Depending on how you explored this website, you might have already come across our page
                        about [cryptographic hash functions](blockchain/hash).
                        What we didn't tell you there is that they are not only used for securing the integrity of the blockchain -
                        they are also used for building the puzzles that nodes use for proving
                        that they worked hard.

                        And the puzzle is the following: Given all the data that you want to include in a block,
                        including the [hash of the previous block](blockchain/hash),
                        find some extra data to append to that block
                        (the so called *nonce*)
                        so that the hash of the block gets a certain number of zeroes at the end.
                        The number of zeroes describes the *difficulty target* - the more they are,
                        the harder it gets.
                        Why?
                        Because the only way to solve this puzzle is to try out many, many nonces...

                        But enough theory. Why don't you try it yourself?
                        The example below is similar to the one above.
                        But now you only control one of the nodes and you are only allowed to publish a block if you
                        have solved the puzzle for a mere 8 zeroes.
                        Oh and by the way the node on the right is also puzzle-solving...
                        "#
                    }
                }
            </p>
            <div class="block">
                <PowExample />
            </div>
            <p class="block">
                {
                    indoc_markdown_content! { r#"
                        And these are the very basics behind Proof-of-Work as it is used by
                        Bitcoin and comparable blockchain systems.
                        What we didn't cover, among others, is that the difficulty of finding
                        that next block is adapted over time.
                        We also didn't discuss the many criticisms that can be voiced against
                        PoW-based systems, such as their huge energy consumption.
                        You might find pointers for further study in the ["Beyond"](beyond) section.
                        "#
                    }
                }
            </p>
        </StandardPage>
    }
}

#[function_component(NoPowExample)]
fn no_pow_example() -> Html {
    let mut sim = isds::Simulation::new_with_underlay_dimensions(200., 50.);
    sim.add_event_handler(isds::InvokeProtocolForAllNodes(
        isds::nakamoto_consensus::NakamotoConsensus::new(),
    ));

    let left_node = sim.spawn_random_node_at_position(0., 0.);
    let right_node = sim.spawn_random_node_at_position(200., 0.);
    let middle_node = sim.spawn_random_node_at_position(100., 50.);
    sim.add_peer(left_node, middle_node);
    sim.add_peer(right_node, middle_node);

    // little hack to make sure that middle_node is initialized
    sim.add_peer(middle_node, left_node);
    sim.remove_peer(middle_node, left_node);

    let sim = sim.into_shared();

    let on_button = |node| {
        let sim = sim.clone();
        Callback::from(move |_| {
            sim.borrow_mut()
                .do_now(MineBlockWithOneRandomTransaction(node))
        })
    };

    html! {
        <isds::Isds sim={ sim.clone() }>
            <div class="columns">
                {
                    [left_node, right_node].iter().map(|&node| html! {
                        <div class="column">
                            <div class="box">
                                <isds::BlockchainView
                                    viewing_node={ Some(node) }
                                    max_visible_blocks={ 4 }
                                    show_unconfirmed_txes={ false }
                                    highlight_class={ "has-fill-info" }
                                />
                                <div class="has-text-centered p-5">
                                    <button
                                        class="button"
                                        onclick={ on_button(node) }
                                    >
                                        { "Propose a block for free!" }
                                    </button>
                                </div>
                            </div>
                        </div>
                    }).collect::<Html>()
                }
            </div>
            <div class="columns is-centered">
                <div class="column is-half-desktop">
                    <div class="box">
                        <isds::NetView
                            toggle_edges_on_click={ false }
                            node_highlight_on_hover={ true }
                            highlight_class={ "has-fill-info" }
                            buffer_space=25.
                        />
                    </div>
                </div>
            </div>
            <div class="columns is-centered">
                <div class="column is-two-thirds-desktop">
                    <div class="box">
                        <isds::BlockchainView
                            viewing_node={ Some(middle_node) }
                            show_unconfirmed_txes={ false }
                            highlight_class={ "has-fill-info" }
                        />
                    </div>
                </div>
            </div>
        </isds::Isds>
    }
}

struct PowExample {
    sim: isds::SharedSimulation,
    left_node: isds::Entity,
    middle_node: isds::Entity,
    left_node_block_data: String,
}
impl Component for PowExample {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        let mut sim = isds::Simulation::new_with_underlay_dimensions(200., 50.);
        sim.add_event_handler(isds::InvokeProtocolForAllNodes(
            isds::nakamoto_consensus::NakamotoConsensus::new(),
        ));

        let left_node = sim.spawn_random_node_at_position(0., 0.);
        let right_node = sim.spawn_random_node_at_position(200., 0.);
        let middle_node = sim.spawn_random_node_at_position(100., 50.);
        sim.add_peer(left_node, middle_node);
        sim.add_peer(right_node, middle_node);

        sim.do_now(isds::AtRandomIntervals::new(
            MineBlockWithOneRandomTransaction(right_node),
            isds::SimSeconds::from(10.),
        ));

        // little hack to make sure that middle_node is initialized
        sim.add_peer(middle_node, left_node);
        sim.remove_peer(middle_node, left_node);

        let sim = sim.into_shared();

        let left_node_block_data = random_block_data();

        Self {
            sim,
            left_node,
            middle_node,
            left_node_block_data,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <isds::Isds sim={ self.sim.clone() } >
                <div class="columns">
                    <div class="column">
                        <isds::HashBox
                            existing_data={ self.left_node_block_data.clone() }
                            show_hex={ false }
                            show_only_last_32_bits={ true }
                            trailing_zero_bits_target={ 8 }
                            highlight_trailing_zero_bits={ true }
                            block_on_reached_target={ true }
                        >
                            <div class="has-text-centered p-5">
                                <div class="notification is-primary">
                                    { "Puzzle solved!" }
                                </div>
                                <button
                                    class="button"
                                    onclick={ ctx.link().callback(move |_| ()) }
                                >
                                    { "Propose block!" }
                                </button>
                            </div>
                        </isds::HashBox>
                    </div>
                    <div class="column">
                        <div class="box">
                            <isds::BlockchainView
                                viewing_node={ Some(self.left_node) }
                                max_visible_blocks={ 4 }
                                show_unconfirmed_txes={ false }
                                highlight_class={ "has-fill-info" }
                            />
                        </div>
                    </div>
                </div>
                <div class="columns is-centered">
                    <div class="column is-half-desktop">
                        <div class="box">
                            <isds::NetView
                                toggle_edges_on_click={ false }
                                node_highlight_on_hover={ true }
                                highlight_class={ "has-fill-info" }
                                buffer_space=25.
                            />
                        </div>
                    </div>
                </div>
                <div class="columns is-centered">
                    <div class="column is-two-thirds-desktop">
                        <div class="box">
                            <isds::BlockchainView
                                viewing_node={ Some(self.middle_node) }
                                show_unconfirmed_txes={ false }
                                highlight_class={ "has-fill-info" }
                            />
                        </div>
                    </div>
                </div>
            </isds::Isds>
        }
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        // there is only one message type...

        self.sim
            .borrow_mut()
            .do_now(MineBlockWithOneRandomTransaction(self.left_node));
        self.left_node_block_data = random_block_data();

        true
    }
}

#[derive(Debug, Clone)]
pub struct MineBlockWithOneRandomTransaction(pub isds::Entity);
impl isds::Command for MineBlockWithOneRandomTransaction {
    fn execute(&self, sim: &mut isds::Simulation) -> Result<(), Box<dyn Error>> {
        random_transaction(sim, self.0);
        sim.do_now(isds::ForSpecific(
            self.0,
            isds::nakamoto_consensus::MineBlock,
        ));
        Ok(())
    }
}

fn random_block_data() -> String {
    format!(
        "{:x}{:x}{:x}{:x}",
        rand::random::<u64>(),
        rand::random::<u64>(),
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}
