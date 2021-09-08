# Battleship

simple, ugly battleship game in rust.

Mostly built as an exercise, but it's playable.

## What'd I learn?

- graphics / rendering (winit + pixels)
    - hadn't really done direct draw-on-the-screen stuff much before. It's annoying!
- rendering text (rusttype and the pixels framebuffer)
- std::net's TcpStream and TcpListener, for sending bytes to another player over
    the network, sort of
    - plus serde + ron for serializing
- rand and such for game logic
- some game programming patterns stuff (command pattern, game loops)
- as always, more practice doing normal rust stuff (organizing structs, enums,
    borrowchecker)

## Questions

The data model for the program. It seems... okay? but maybe not good. What would
good look like? What about the code structure?

I have this big `World` struct. It helps to prevent passing around tons of args to
every function (though the `frame` arg still gets passed everywhere). But is it
a good pattern, or is it bad in some way?

how 2 do network stuff good instead of bad
