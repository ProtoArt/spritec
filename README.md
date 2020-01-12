# spritec - The sprite compiler

This project is currently in the **prototyping** phase. That means that while
we have a working proof-of-concept, we are not yet ready for anyone to start
using our software.

**To be notified of our progress, please sign up on our website: [protoart.me](https://protoart.me/)**

## Initial Prototype

One of the hard parts of using hand-drawn assets is keeping them up to date. You
might have a spritesheet with 25 different sprites and if you decide to change
one detail you then have to go and do that 25 different times.

Our idea is to start at a higher level of abstraction. Instead of a 2D
spritesheet, we take a 3D model and render it at different angles in different
poses. That means that instead of modifying 25 different sprites, **you only have
to change ONE 3D model to update *everything***.

Eventually, we would like to make it so you don't even need to bring your own 3D
model. You can just "program your art" and get the sprites you need to start
your game.

> **Note:** While we came up with this idea on our own, it certainly isn't new
> or unique. In fact, the very popular Dead Cells game [used this exact
> technique][dead-cells-game-art] to create the art for their game.
>
> **That's great!** It means that this idea has already been shown to work well
> in a very large scale game.

[dead-cells-game-art]: https://www.gamasutra.com/view/news/313026/Art_Design_Deep_Dive_Using_a_3D_pipeline_for_2D_animation_in_Dead_Cells.php

## Example

Suppose you had the following 3D model:

![bigboi render](https://raw.githubusercontent.com/ProtoArt/spritec/5a345767306c246ca88170594249200101029f34/samples/bigboi/render/bigboi.png)

You could rig up the model with an armature/skeleton to pose it however you
want:

![bigboi posed](https://raw.githubusercontent.com/ProtoArt/spritec/5a345767306c246ca88170594249200101029f34/samples/bigboi/render/bigboi_rigged.png)

Then you can animate it so that it looks like it's walking:

![bigboi walk](https://raw.githubusercontent.com/ProtoArt/spritec/5a345767306c246ca88170594249200101029f34/samples/bigboi/render/bigboi_rigged_walk.gif)

Running this through `spritec` would give you a preview window that shows this
same walking animation turned into pixel art:

![bigboi pixel art walking](https://raw.githubusercontent.com/ProtoArt/spritec/5a345767306c246ca88170594249200101029f34/samples/bigboi/render/bigboi-walking.gif)

You can then export that from the software as either individual pixel art
frames, or as a finished spritesheet:

![bigboi pixel art walking sprites](https://raw.githubusercontent.com/ProtoArt/spritec/5a345767306c246ca88170594249200101029f34/samples/bigboi/render/bigboi_spritesheet.png)

Getting a side view is as simple as changing the angle we render from:

![bigboi pixel art walking sprites side](https://raw.githubusercontent.com/ProtoArt/spritec/5a345767306c246ca88170594249200101029f34/samples/bigboi/render/bigboi_spritesheet_side.png)

**That's all it takes!** From one model you can potentially derive tens, if not
hundreds, of different sprites for every frame of every animation you want.
This frees you up to iterate and tweak your designs without having to worry
about redrawing everything.

One of the things you'll see is that you don't even need very sophisticated 3D
models to generate pixel art. Pixel art games usually use very small tile sizes
like 32x32 or 64x64. There isn't a lot of room for detail there. That makes this
art style perfect for this technique. The art is a little more crude, but the
medium allows for that so it's okay.

## Project Goals

The goal of our project is to make creating pixel art spritesheets easier and
faster for indie game developers. Games that are made by a single developer or a
very small team may not have the resources or skill to create the art they are
going for right as they start their project. There are plenty of online game
assets, but all of them are limited in how much they can be customized or added
to.

We want to make a suite of tools that make creating game art more accessible to
programmers and developers who need help making their art. The art they create
with our tools can either last throughout the development of their game or
eventually get replaced if/when they hire a professional artist.

While we do not aim to replace the role of artists in game development, we are
hoping to create high quality art that can help developers who are starting
their games get to their vision faster.

## Subscribe

**This is just the beginning!** Sign-up on our website to be notified of our
progress: [protoart.me](https://protoart.me/). We are so excited to help make
creating game art easier for developers!
