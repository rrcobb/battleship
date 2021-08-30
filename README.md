# Battleship

- hits / misses on ships
    - for now: copy other player from this player, effectively shooting at
        yourself
- draw text
    - DONE choose a library!
    - DONE draw title
    - add help text
        - DONE Place your ships!
          - arrow keys to move
          - space to rotate
          - return to place
        - DONE Take aim
        - Waiting for your opponent to (place their ships)
        - Waiting for your opponent to (fire)
        - Miss...
        - Hit!
        - sunk ship messages
        - Hit! You sunk their x
    - add grid labels
    - add 'score' - ships remaining

- local version: how to play against an AI or something?
- local version: play on the same screen
- game mode selector
- prevent ships from locking when they intersect
- movement rerendering is... slow somehow?
- internet version
- Build for windows, mac, web?
- web server for:
  - get latest binary
  - find active players to start a game
- nice to have: bmps for the ships
- sounds with https://docs.rs/rodio/0.14.0/rodio/ or something
- how to confirm done?
- for now, just.. when the last ship is placed
- how to edit after placed?

- ship placement / setup
  - DONE move boats with arrow keys
  - DONE confirm with key
  - DONE select which boat to move somehow (iteratively)
  - DONE turn ships
- DONE ship status (placing, locked, hidden)
- figure out turn taking
    - DONE move target with arrow keys
    - DONE select with space or enter
    - DONE adds to shots taken
- get grids rendering per player
    - DONE rendering empty grids
    - DONE render ships on grids
        - but, actually the ships instead of just squares
