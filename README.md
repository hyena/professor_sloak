Professor Sloak
===============

This is a silly slack bot written to practice Rust.

The bot wouldn't be possible without data from the [veekun pokedex]
(https://github.com/veekun/pokedex). Thank you, Eevee!


Usage
-----

 1. Create an API key for your bot and invite it to the desired channels.
 2. `cargo run -- <api-key>`
 3. Type `#pokeme` in slack to meet your new spirit animal.

The bot as written is designed to try to reconnect indefinitely with a
5 second cooldown between attempts.


Known Issues
------------

The bot is hardcoded to use English language text.
