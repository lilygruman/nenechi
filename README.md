# Nene

This is a basic 2.5D physics maze game I built in
[Bevy](https://bevyengine.org/).
It uses the
[Avian](https://github.com/Jondolf/avian)
physics engine.

## Name and Inspiration

I named it after Nene Sakura (桜 ねね), a character from the 2016-17 anime
[*New Game!*](https://en.wikipedia.org/wiki/New_Game!) whom,
after joining the game development company Eagle Jump as a developer-trainee,
is given the task of creating a
[maze game](https://en.wikipedia.org/wiki/Ball-in-a-maze_puzzle)
like this one as a training exercise (S2E9-10).
I realized that this was a totally attainable goal for me as well,
so I went ahead and gave it a try. I suppose *Nene Quest* is next...

## Features

I originally tried more 3D controls and physics, but there were too many design decisions to make
in order to manage the camera and keep the ball from bouncing out of the maze,
so for now I have it stood up vertically, with a fixed camera, orthogonal rotation,
and an invisible cover to keep the ball in the maze.

The controls are currently:
* Left analog stick x-axis to rotate the maze
* Start button to reset the ball
* Triggers (R2/L2) to move the camera forward and backward
* Bumpers (R1/L1) to change the field of view; press both simultaneously to reset

The camera actions are vestigial from when I was working in 3D,
but they don't hurt anything so I left them in.

Other missing features:
* Win condition
* Different mazes
  * Random generation
  * Different dimensions
  * Different types (e.g. unicursal, 3D)
* Art/design elements, e.g. sound, textures/materials, lighting
* Show controls on screen
