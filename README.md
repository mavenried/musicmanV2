<h1 align="center">
musicmanV2
</h1>
<p align="center">
A music server implementation in Rust. Learning project for tokio::tcp
and async programming in general. Beefed-up version of its predecessor,
musicman.
</p>

# musicman-server

## Installation

    cargo install musicman-server

Starts on port `4000`.\
You can also pass in a port number.

# musicman-client

## Installation

The recommended way is to build from source or install using Cargo.

    cargo install musicman-client

Attempts to connect to `0.0.0.0:4000` by default.\
You can pass in a custom address.

## Commands

Running musicman presents you with a prompt:

    $ musicman
    musicman❯

## Available Commands

### clear

Clears the queue.

    clear

### show / ls

Show the current queue.

    show
    ls

### search

Adds matches to your queue.

    search [artist|title] <search_term>

Will prompt you to choose which result to add to the queue.

### replay

Replay the current song.

    replay

### pause / p

Pause/resume playback.

    pause
    p

### next

Skip to the next song in the queue. Optionally takes the number of
tracks to skip.

    next
    next <n>

### prev

Skip to the previous song in the queue. Optionally takes the number of
tracks to skip.

    prev
    prev <n>

### playlist / pl

Playlist creation and playback.

#### new

Save the current queue as a playlist.

    playlist new <name>

#### show / ls

Show all available playlists.

    playlist show
    playlist ls

#### load

Load the specified playlist as the current queue.

    playlist load <name>

### exit

Quit the player.

    exit
