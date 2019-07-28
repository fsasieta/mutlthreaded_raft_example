# Implementation of a simulated multithreaded environment using the raft algorithm
I tried to tackle this in a constrained time frame, having zero experience with the
algorithm and not being knowledgeable with any of the supporting apis.

There were examples in the current version of the code that were helpful, but they
assumed familiarity with the algorithm and the documentation (were there was any)
did not link the interface with the algorithm itself. Also, it has been a while
since I had done protobufs, but thankfully I only had to use the interfaces for them.

The talk given by Siddon Tang located [here](https://youtu.be/MSrcdhGRsOE) was
extremely helpful in understanding how the library and the algorithm worked more
or less.

Relying heavily on examples I fear my own version may not be too different from
the ones provided in the library, but that is for some else to judge.

## Notes on my implementation:
1) It doesn't work
2) It panics on the consensus portion. Haven't finished the implementation of it
3) It is only able to work using the public alpha version of the latest raft library.

I'm using the 0.6.0-alpha version of the raft crate because any other version 
was impossible to use due to the bug located [here](https://github.com/pingcap/raft-rs/pull/200). Or
at the very least I'm unaware as to how can an interface be implemented without the
ability to satisfy the objects by it. For more exploration, try to fulfill the interface
of the "Storage" trait using the 0.5.0 (or any other lower version). You will be unable to 
do so because you will be unable to create a "RaftState" object. 

I have to admit I spent way to much time figuring this out. And I stopped when figuring this out.
It would have been great to have a heads up for this. In case the bug is unclear, I added an example
[here](https://github.com/fsasieta/example_raft_rs_bug)

The reason the current examples on the 0.5.0 version work is because they are on the
same scope as the declaration of the struct. The only way to get access to the 0.6.0
