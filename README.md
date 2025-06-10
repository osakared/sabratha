# 🧜🏽‍♀️ Sabratha

[![A mushroom-head robot drinking bubble tea](/assets/images/codey.jpg 'Codey the Codecademy mascot drinking bubble tea')](https://codecademy.com)


[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/osakared/sabratha/workflows/CI/badge.svg)](https://github.com/osakared/sabratha/actions?query=workflow%3ACI+branch%3Amain)
[![Discourse Topics](https://img.shields.io/discourse/topics?server=https%3A%2F%2Fsupport.tripoli.app)](https://support.tripoli.app)

The rust core of Tripoli, used for datalogging/machine control. Responsible for the data model at the lowest level and ultimately should be usable as embedded code on devices.

## Basic (planned) Architecture

Each instrument connection is its own process, with its own arrow-rs db to store rows of data, if applicable. The main process handles forwarding requests to the others and spawning or in embedded contexts, the main process handles both message handling and one instrument connection.

There is flexibility in this in that any process that adheres to the basic communication protocol could function as an instrument connection, even if it's not a sabratha instance.



