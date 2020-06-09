Ticket to Ride Protocol
===

An attempt to reverse engineer and eventually implement the network protocol used by
Days of Wonder's Ticket to Ride mobile game.

Protocol
---
So far, all I've determined is that the game servers advertise themselves using
mDNS with the type string "_t2rdaysofwonder._tcp".
I'm currently attempting to man-in-the-middle a game run between two phones to capture the actual
game protocol.

### Mitm
When running the game through a MitM server, each side sends an initial identification packet,
but no data is sent after that.
Using my laptop to create a Wi-Fi access point, I was able to let the instances connect to each other
directly while still getting a pcap through Wireshark.
Looking at the pcap, the initial identification packets are sent, and then the connecting client is supposed
to send another packet containing information such as UUID.

Looking at the packets, they seem to be composed of:
- 32-bit big endian packet type id
- 32-bit big endian length field
- A protobuf encoded struct

I don't have the .proto files, which makes determining what the protobuf fields mean difficult.
I'm able to use `protoc --decode_raw` at least determine the structure.
The opening message has type id 3 and looks like

Connecting player:
```
2: "name"
3: 3923117471172131697
4: "myContext"
```

Server player:
```
2: "sean"
3: 15632285327718950011
4: ""
```

If attempting to MITM, this is the end of communication other than heartbeats (empty, type id 2).
When the games connect directly to each other, the connecting player sends a message after that has type id 1 and looks like:
```
1 {
  1: "name"
  2: "da5d37a9-d5e7-406b-8529-6ed435c26655"
  3: 18446744073709551615
  4: 1
}
```

Using Ghidra, the opening message is named `LocalGameGenericConnect`.
There's a field named PeerId.  This is probably some hash which is failing validation when mitm'ing.
Searching gives the LocalPeerIdGeneric struct, which has a field named displayName, and a randomly generated
64 bit int, which looks like the first and second fields in the connect message.
It also has a function \_buildServiceName, which sounds like it's related to the MDNS service name.
The MITM is probably failing when the connecting player realizes that the expected service name is not
the same as the one it actually found.

It seems the service name string is just the random peer id converted to a string in base 36.
At this point I can start writing a slightly more active MITM.

## Randomization
It appears randomization is handled in a distributed manner, where only the seed is shared in advance
and each system keeps its state intact.

This makes things much more difficult because it means an AI player would need to replicate this RNG.

## Train cards
It seems like each card as a unique ID, presumably from 0-109.
Ids 39, 42, 45 are blue, so blue is probably id 4 ([36,48)).
104 and 106 are rainbox, so they're the top of the range.
84 and 86 are green, id 7.
