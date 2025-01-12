# pine64-neigbourhood-watch

Simple watchdog daemon for a single-board-computer based cluster. This was developed for a cluster of 6 ARM based Pine64 SBCs.

The daemon is monitoring connectivity to the neighbours by periodically pinging them (ICMP echo-request) and resets its immediate nighbour if it looses connectivity to it. To reboot the neigbour the daemon expects that one of its GPIO pins is connected to the (hard-) reset pin of the immediate neigbour.
