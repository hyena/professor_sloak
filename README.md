Professor Sloak
===============

This is a silly slack bot written to practice Rust.


Usage
-----

 1. Create an API key for your bot and invite it to the desired channels.
 2. `cargo run -- <api-key>`
 3. Type `#pokeme` in slack to meet your new spirit animal.

Known Issues
------------

The bot uses (veekun)[https://www.veekun.com] for its pokemon art images.
Since veekun doesn't yet have art for all the new pokemon, the bot
cuts off the number of maximum pokemon it generates.

The bot is hardcoded to use English language text.
