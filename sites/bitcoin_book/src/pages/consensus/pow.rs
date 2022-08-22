use super::*;
pub use std::error::Error;

#[function_component(Pow)]
pub fn pow() -> Html {
    html! {
        <>
        <Header title="Proof-of-Work (PoW)">
            {
                indoc_markdown_content! { r#"
                    Bitcoin, as well as most other cryptocurrency networks,
                    is what is called a *permissionless* system.
                    This means that anyone can become an active part of the network and,
                    for example,
                    propose new blocks.
                    No participant needs to register anywhere or ask for permission.
                    Between the lines, this means that we don't really know who is
                    responsible for the messages that fly through the network.
                    It also means that it's hard to do something like *voting*.
                    We have a problem if we want to decide democratically what should be the correct next block,
                    for example.
                    There is no way to enforce that each participant gets one vote,
                    or even a limited number of votes.

                    Bitcoin's solution is to tie block creation to *work*.
                    The underlying assumption is that the ability to do work is more or
                    less evenly distributed.
                    But let's take a closer look at the problem first...
                    "#
                }
            }
        </Header>
        <Section>
            <h3 class="title is-4">{ "Naive example: blocks for free" }</h3>
            <div class="block">
                {
                    indoc_markdown_content! { r#"

                        In the example below, the top nodes are competitors,
                        each tries to convince the third node that *its* chain is the correct one.
                        The example uses the *longest chain rule* -
                        the bottom node will assume that the longest chain it knows is the correct one.
                        However, creating new blocks is really easy in this example...
                        Try it!
                        Then try confusing the bottom node!
                        "#
                    }
                }
            </div>

            <div class="block">
                <NoPowExample />
            </div>
            <div class="block">
                {
                    indoc_markdown_content! { r#"
                        As you may have noticed, it's easy to arrive in a situation where honest nodes
                        have to continuously revise their opinion about what the "true" history is
                        (the [blockchain](blockchain) is basically a logbook).
                        This is not great - imagine the `coin` balance in your wallet fluctuating all the time
                        because the system can't make up its mind about what happened.
                        "#
                    }
                }
            </div>
        </Section>
        <Section>
            <h3 class="title is-4">{ "We'll have to work then..." }</h3>
            <div class="block">
                {
                    indoc_markdown_content! { r#"
                        This is where Proof-of-Work (PoW) comes in.
                        PoW means that a node needs to prove that it worked really hard on that new block.
                        Only then is it allowed to add that block to the chain.

                        What is work?
                        Unfortunately it can't be anything that is intrinsically valuable...
                        In the context of Bitcoin's PoW, *work* is essentially: *solving puzzles*.

                        (This puzzle-solving is also known as *mining*.
                        Nodes get to create new `coins` in each block they create,
                        i.e., for each puzzle they solve.
                        They are *mining* these `coins`...)

                        But what kind of puzzles are we talking about?
                        Depending on how you explored this website, you might have already come across our page
                        about [cryptographic hash functions](blockchain/hashes).
                        What we didn't tell you there is that they are not only used for securing the integrity of the blockchain -
                        they are also used for building the puzzles that ensure that nodes are working hard.

                        And the puzzles look like this: Given all the data that you want to include in a block,
                        including the [hash of the previous block](blockchain/hashes),
                        find some extra data to append to that block
                        (the so called *nonce*)
                        so that the hash of the new block gets a certain number of zero bits at the end.
                        (It's actually a tiny bit more complicated than "number of zero bits at the end",
                        but it's a good enough approximation.)
                        The number of zeroes describes the *difficulty target* - the more they are,
                        the harder it gets.
                        Why?
                        Because the only way to solve this puzzle is to try out many, many nonces...
                        Each nonce is like a lottery ticket,
                        nodes check it by calculating a hash with it,
                        and if that hash has enough zeroes at the end then the ticket was a winning ticket.

                        Enough theory. Why don't you try it yourself?
                        The example below is similar to the one above.
                        But now you only control one of the nodes and you are only allowed to publish a block if you
                        have solved *the puzzle*.
                        You need to find a value (in the "Type anything" field) that leads to a hash with enough zeroes at the end.
                        We set the difficulty target to a mere 8 zero bits.
                        That shouldn't be too hard, right?
                        Oh and by the way that other node... it's also puzzle-solving...
                        "#
                    }
                }
            </div>
            <div class="block">
                <PowExample />
            </div>
            <div class="block">
                {
                    indoc_markdown_content! { r#"
                        And these are the very basics behind Proof-of-Work as it is used by
                        Bitcoin and comparable blockchain systems.
                        What we didn't cover, among other things, is that the difficulty of creating
                        new blocks is adapted over time.
                        We also didn't discuss the many criticisms that can be voiced against
                        PoW-based systems, for example that their energy consumption is **HUGE**.
                        You might find pointers for further study in the ["Beyond"](beyond) section.
                        "#
                    }
                }
            </div>
        </Section>
        <Footer />
        </>
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
    right_node: isds::Entity,
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
            right_node,
            middle_node,
            left_node_block_data,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <isds::Isds sim={ self.sim.clone() } >
                <div class="columns">
                    <div class="column">
                        <div class="box">
                            <isds::BlockchainView
                                viewing_node={ Some(self.left_node) }
                                max_visible_blocks={ 4 }
                                show_unconfirmed_txes={ false }
                                highlight_class={ "has-fill-info" }
                            />
                            <div class="pt-5">
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
                        </div>
                    </div>
                    <div class="column">
                        <div class="box">
                            <isds::BlockchainView
                                viewing_node={ Some(self.right_node) }
                                max_visible_blocks={ 4 }
                                show_unconfirmed_txes={ false }
                                highlight_class={ "has-fill-info" }
                            />
                            <div class="py-5">
                                <span class="mr-1">
                                    { "This node is puzzle-solving (AKA "}
                                    <span class="is-italic">{ "mining" }</span>
                                    { ")..." }
                                </span>
                                <isds::Spinner
                                    title={ "Mining in progress..." }
                                    spins_per_second={ 10. }
                                />
                            </div>
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
        "{:02x}{:02x}{:02x}{:02x}",
        rand::random::<u64>(),
        rand::random::<u64>(),
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}
