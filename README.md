# Not Snake

This is a game made in Rust using the awesome [Bevy engine][bevy]. It can be played [here][itch]. 

I started this project with the only goal of completing a game, following two guidelines of "if it ain't broke, don't fix it" and "save it for the next game" in an attempt to progress while preventing scope creep. It helped me move forward and complete the game, but the cost can be clearly seen across the code base.

There is *a lot* that can be improved in this code base and I strongly advise anyone wanting to learn from this code to keep the above in mind. Again, this project was made while learning bevy, ecs concepts and how to make a game in general while also just throwing ideas at the wall and seeing what sticks.

I learned a lot making this game and am getting started on a new game that will (hopefully) be much more idiomatic. I hope pieces of this project can be useful to others with the understanding that my priorities were finishing a game, not making anything reusable. 

I'm always hanging out in the [bevy discord][bevy-discord], definitely feel free to @ramirezmike me and ask questions or criticize me :)

Also, feel free to fork/make issues!


# Running the Game

To run the game locally

```
cargo run --release --features native
```

To run the browser version

```
cargo make serve
```

which will compile and serve the web version at http://127.0.0.1:4000

# Special Thanks
cart - for being a cool dude

alice/IvyLashes - for being really knowledgable and super helpful

[TheRawMeatball][meatball] - for being really helpful a lot

NiklasEi - the [Bevy game template][bevy-template] and [kira audio][audio] are amazing!!

OptimisticPeach - for answering my shader questions that was neat

StarToaster - also for answering my other shader questions 

robswain - your [bevy-hud-pass][bevy-hud-pass] made my hud all cool

[aevyrie][aevyrie] - I think I managed to use all of your plugins

gin - for making really out-of-the-box suggestions

Toqoz - your [line crate][linecrate] helped me fix a ton of bugs

Joy - for helping me learn rotations

Ida Iyes - your [bevy cheatbook][cheatbook] was super super helpful!

and thanks to everyone else in the Bevy discord!


[bevy]: https://bevyengine.org/
[itch]: https://ramirezmike2.itch.io/not-snake 
[bevy-discord]: https://discord.gg/bevy
[bevy-template]: https://github.com/NiklasEi/bevy_game_template
[aevyrie]: https://github.com/aevyrie
[audio]: https://github.com/NiklasEi/bevy_kira_audio
[meatball]: https://github.com/TheRawMeatball
[bevy-hud-pass]: https://github.com/superdump/bevy-hud-pass
[cheatbook]: https://github.com/bevy-cheatbook/bevy-cheatbook
[lincrate]: https://github.com/Toqozz/bevy_debug_lines
