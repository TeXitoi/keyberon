# Keyberon [![Build status](https://travis-ci.org/TeXitoi/keyberon.svg?branch=master)](https://travis-ci.org/TeXitoi/keyberon) [![](https://img.shields.io/crates/v/keyberon.svg)](https://crates.io/crates/keyberon) [![](https://docs.rs/keyberon/badge.svg)](https://docs.rs/keyberon)

A rust crate to create a pure rust keyboard firmware.

It is exposed as a library giving you the different building blocks to
create a featureful keyboard firmware. As the different functionality
are interconected by the user of the crate, you can use only the parts
you are interested in or easily insert your own code in between.

This crate is a no_std crate, running on stable rust. To use it on a
given MCU, you need GPIO throw the [embedded hal
crate](https://crates.io/crates/embedded-hal) to read the key states,
and the [usb-device crate](https://crates.io/crates/usb-device) for
USB communication.

## Projects using this firmware

[List of Repositories using Keyberon](./KEYBOARDS.md)

The first project using this firmware is [Keyberon
grid](https://github.com/TeXitoi/keyberon-grid), a handwired keyboard
with a grid of keys. It is based on the blue pill, a cheap development
board based on a STM32F103 MCU.

![keyberon-grid](https://raw.githubusercontent.com/TeXitoi/keyberon-grid/master/images/keyberon.jpg)

There is a [port](https://github.com/TeXitoi/ortho60-keyberon) to
[Cannon Keys](https://cannonkeys.com/)'s [Ortho60 keyboard
kit](https://cannonkeys.com/collections/frontpage/products/ortho60)
(blue pill based).

![Ortho60](https://cdn.shopify.com/s/files/1/0238/7342/1376/products/Ortho60_1024x1024@2x.jpg)

Another handwired project using keyberon is
[keyberon-f4](https://github.com/TeXitoi/keyberon-f4), a unsplitted
ergo keyboard. It runs on a [WeAct
MiniF4](https://github.com/WeActStudio/WeActStudio.MiniSTM32F4x1) based on a
STM32F401 MCU.

![keyberon-f4](https://raw.githubusercontent.com/TeXitoi/keyberon-f4/master/images/keyberon-44.jpg)

[TssT16](https://github.com/TssT16)'s 4x12 keyboard (blue pill based):

![TssT16's](https://user-images.githubusercontent.com/12481562/81586297-97996e80-93b5-11ea-86e1-c4358854477e.jpg)

[gilescope](https://github.com/gilescope)'s 4x12 keyboard (keyberon
grid, blue pill based):

![gilescope's](https://i.redd.it/syvlwmkd77851.jpg)

[covah901](https://www.reddit.com/user/covah901/)'s keyboard ([WeAct
MiniF4](https://github.com/WeActStudio/WeActStudio.MiniSTM32F4x1) based):

![covah901](https://i.redd.it/gnkfymu0gwo41.jpg)

[KeySeeBee](https://github.com/TeXitoi/keyseebee), a split ergo
keyboard (STM32F072 based).

![KeySeeBee](https://raw.githubusercontent.com/TeXitoi/keyseebee/master/images/keyseebee.jpg)

[Arisu handwired](https://github.com/help-14/arisu-handwired) using STM32F401.

![Arisu handwired](https://camo.githubusercontent.com/4fca994ac2b7c1b1874d4331c2428cac211ff80c2891c75c971d15630ef0a948/68747470733a2f2f692e696d6775722e636f6d2f30334c356f63702e6a7067)

## Features

The supported features are:
 - Layers when holding a key (aka the fn key). When holding multiple
   layer keys, the last pressed layer action sets the layer.
 - Transparent key, i.e. when on an alternative layer, the key will
   inherit the behavior of the default layer.
 - Change default layer dynamically.
 - Multiple keys sent on an single key press. It allows to have keys
   for complex shortcut, for example a key for copy and paste or alt tab, or
   for whatever you want.
 - Chording multiple keys together to act as a single key
 - hold tap: different action depending if the key is held or
   tapped. For example, you can have a key acting as layer change when
   held, and space when tapped.
   

## FAQ

### Keyberon, what's that name?

To find new, findable and memorable project names, some persons in the rust community try to mix the name of a city with some keyword related to the project. For example, you have the [Tokio project](https://tokio.rs/) that derive its name from the Japanese capital Tokyo and IO for Input Output, the main subject of this project.

So, I have to find such a name. In the mechanical keyboard community, "keeb" is slang for keyboard. Thus, I searched for a city with the sound [kib], preferably in France as it is the country of origin of the project. I found [Quiberon](https://en.wikipedia.org/wiki/Quiberon), and thus I named the project Keyberon.
