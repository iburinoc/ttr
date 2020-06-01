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
