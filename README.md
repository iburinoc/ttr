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
  - I don't have the .proto files, which makes determining what the protobuf fields mean difficult.
    I'm able to use `protoc --decode_raw` at least determine the structure.
