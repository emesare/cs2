# Source 2 (cs2) [Bevy] mod framework

What happens when you try and take a closed source game engine and tie it to a modular game engine written in a completely different language with a different engine architecture? Well you get whatever this is.

## NOTICE

This really is far from that. I have yet to hoist the game engine into the [Bevy ECS](https://bevyengine.org/learn/book/getting-started/ecs/), however thats not the hard part really. The difficult part is making it not lock the render thread, more on that later... *hopefully*.

## Why?

To show it is easier then you would think, take a look at how the Bevy game engine is [organized](https://github.com/bevyengine/bevy/tree/main/crates), notice how everything is a Plugin on top of the [ECS](https://bevyengine.org/learn/book/getting-started/ecs/), allowing for core components to easily be swapped out, such as the renderer. Just don't expect third-party plugins to be as nice, many depend on bevy_render which ultimately is unfit for a modding framework... ***unless*** you want to swap out the games entire rendering.

[Bevy]: https://bevyengine.org/