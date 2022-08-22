Consensus is the process by which the network decides which chain of blocks is going to be the "correct" blockchain.
It's what gives nodes confidence that the blockchain they are seeing is the blockchain that everybody else is seeing.
Consensus in Bitcoin is characterized by two central features:

- Use of [Proof-of-Work](consensus/pow) (an extremely energy-intensive lottery) for determining who will get to propose (or *mine*) the next block.
- Use of the *longest chain rule* for determining the true blockchain whenever there are multiple options,
for example in the event of *forks*.

The specifics are a bit complex - we hide them behind a spinning wheel and a magic button here!
Click on the button to cause a random node to "win the lottery" and mine a block.

Or just wait for ~10 minutes of simulated time - this is the average *block interval* in Bitcoin,
the average time it takes for the network to mine a block.
