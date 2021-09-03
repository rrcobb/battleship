# Battleship

- layer of indirection for moves, to support alternate game modes (command
    pattern)
    - struct "move" that a player / world consumes?
- starting screen
- game mode selection
    - ai level
    - ship length? (short ships vs original length ships)
    - eventually, internet vs local ai
  - store the hits on the structs instead of recalculating every time
- play vs computer
  - 'smarter' / harder AI
      - doesn't know your ships, but is smarter about where to shoot
- messages for hit/miss log
  - Waiting for your opponent to (place their ships)
  - Waiting for your opponent to (fire)
  - Miss...
  - Hit!
  - sunk ship messages
  - Hit! You sunk their x
    - requires ship names
  - 'score' - ships remaining, per player

- local version: play on the same screen against another person
  - hide placement from each other
  - just targeting / hits / misses
- design
  - improved colors
  - target red should be a different shape (target?)
  - better text sizing / placement
- internet / local network version
    - see only your ships
    - try to hit the other player
- allow mouse to select
- ship placement
  - allow edit ships after placed
  - allow undo for placing ships
- movement rerendering is... slow somehow?
    - is this waiting on inputs too slowly?
- refactors and perf improvements
    - keep more state, do less compute
      - ship status
      - shot status (hit or miss)
      - overlaps - change data structure?
    - don't redraw the 'entire' world every time
        - track changes?
    - keep a rendered font gpu cache
  - should things like font and rng use lazy-static globals?

- Build for windows, mac, web
- web server for:
  - get latest binary
  - find active players to start a game
- nice to have: .bmp's for the ships
- sounds with https://docs.rs/rodio/0.14.0/rodio/ or something

- DONE ship placement / setup
  - DONE move boats with arrow keys
  - DONE confirm with key
  - DONE select which boat to move somehow (iteratively)
  - DONE turn ships
  - DONE prevent ships from locking when they intersect
- DONE ship status (placing, locked, hidden)
- DONE(ish, for now) figure out turn taking
    - DONE move target with arrow keys
    - DONE select with space or enter
    - DONE adds to shots taken
- DONE get grids rendering per player
    - DONE rendering empty grids
    - DONE render ships on grids
- DONE render hits / misses on ships
    - DONE for now: copy other player from this player, effectively shooting at yourself
    - DONE: AI: random move of target
        - DONE don't allow firing again at an existing shot
- DONE random ship placement
- DONE draw text
- DONE grid labels
    - DONE Place your ships!
    - DONE Take aim
    - DONE choose a library!
    - DONE draw title
- DONE game end (when all ships are sunk)

## thinking through multiplayer

- each side keeps full game state
- sends 'moves' to the other side
    - update AI to send a stream of moves
- responds to incoming moves by updating 'other player'
  - both in 'placing' and in 'playing'? probably
- both sides implement same play logic

TODO:
- DONE turn virtualkeycodes into "move" objects
    - DONE,sorta keep a vec of 'moves' to process?
- DONE update the handlers to act based on move objects instead of virtual
    keycodes
- DONE update ai player to emit moves instead of change its state directly
- introduce another player 'kind' that gets its moves from a tcp
    connection
    - possibly: put the tcp handling on another thread, and buffer the inputs
- on game end, restart again with the same player
