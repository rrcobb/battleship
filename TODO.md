# TODO

- info messages
  - Miss...
  - Hit!
  - sunk ship messages
  - Hit! You sunk their x
    - requires ship names
  - 'score' - ships remaining, per player
  - Waiting for your opponent to (place their ships)
  - Waiting for your opponent to (fire)
     - (will need some state for this I think)
  - show current game type (plus connection info?)
- play vs computer
  - 'smarter' / harder AI
      - doesn't know your ships, but is smarter about where to shoot
    - update AI to send a stream of moves

## thinking through multiplayer

- figure out who the host is, and make one player start first, and the other
    second
- DONE don't send empty vecs across the wire
- add liveness info, handle connection errors gracefully
    - send 'I'm alive' pings
    - poll instead of blocking wait for opponent
    - attempt to reconnect if connection lost
- use relay server to share moves between clients, instead of making a direct
    connection
    - like... just http?
- move tcp stuff to a thread
- on game end, restart again with the same player?

## More TODOs:

- More settings
    - ai level
    - ship length? (short ships vs original length ships)
- local version: play on the same screen against another person
  - hide placement from each other
  - just targeting / hits / misses
- design
  - improved colors
  - target red should be a different shape (target?)
  - better text sizing / placement
  - maybe, hide the grids when they aren't in use, instead of showing them at
      all times
  - maybe render just one grid, and allow player to switch to looking at their
      own grid at will
  - label ships / grids
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
    - keep a rendered font gpu cache
    - keep more state, do less compute
      - ship status
      - shot status (hit or miss)
      - overlaps - change data structure?
    - store the hits on the structs instead of recalculating every time
    - don't redraw the 'entire' world every time
        - track changes?
  - should things like font and rng use lazy-static globals?

- Build for windows, mac, web
- web server for:
  - get latest binary
  - find active players to start a game
- nice to have: .bmp's for the ships
- sounds with https://docs.rs/rodio/0.14.0/rodio/ or something

## DONE

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
- DONE starting screen
- DONE game mode selection
    - DONE internet vs local ai
- DONE layer of indirection for moves, to support alternate game modes (command pattern)
    - DONE struct "move" that a player / world consumes
- DONE turn virtualkeycodes into "move" objects
    - DONE, sorta: keep a vec of 'moves' to process?
- DONE update the handlers to act based on move objects instead of virtual
    keycodes
- DONE update ai player to emit moves instead of change its state directly
- DONE refactor 'Move' struct and variables to a different name to avoid
    conflict with the rust keyword
- DONE each side keeps full game state
- DONE sends 'moves' to the other side
- DONE responds to incoming moves by updating 'other player'
- DONE both sides implement same play logic
- DONE when game type is network, need to: broadcast the moves to the tcp connection
  - DONE and get moves from the tcp connection from the other player
- DONE update other player's ships
- DONE figure out how to share ship positions after placed
