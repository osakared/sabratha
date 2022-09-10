# Sabratha

The rust core of Tripoli, used for datalogging/machine control. Responsible for the data model at the lowest level and ultimately should be usable as embedded code on devices.

## Basic (planned) Architecture

Each instrument connection is its own process, with its own sqlite (or other) db to store rows of data, if applicable. The main process handles forwarding requests to the others and spawning or in embedded contexts, the main process handles both message handling and one instrument connection.

There is flexibility in this in that any process that adheres to the basic communication protocol could function as an instrument connection, even if it's not a sabratha instance.