At the end of the day, a Bitcoin-like system is a bunch of computers talking to each other.
These computers form what is called a *peer-to-peer network* -
a network formed when equally important nodes (peers) start forming links between each other.
Having a *link* between two peers simply means that the peers know each others' addresses and can send messages to each other over the Internet.

#### Flooding <i class="fas fa-bullhorn"></i>

The arguably most important service that the peer-to-peer network provides in a Bitcoin-like system is
*broadcast* - getting all peers in the network to receive a certain message.
We need this so that everyone becomes aware of new blocks on the blockchain, for example.

Unlike your favorite radio station,
computers on the Internet cannot simply broadcast their information to everyone that is interested -
only point-to-point communication is well supported.

We need to come up with a workaround, and that workaround is called *flooding*:
We tell each of our peers about some information, and they then do the same.
Eventually, the information spreads to all nodes.

Try it!
Click on one of the nodes in the network,
which will cause it to create a random transaction and flood it.
You might want to activate "Slow down on messages" first (upper right corner) so that you actually get to see the messages.

A bigger network to play with is available [here](network).
