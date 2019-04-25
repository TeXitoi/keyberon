# Building instructions

## Shopping list

For this project, you'll need
 - 60 Cherry MX compatible switches;
 - 60 1u keycaps;
 - 60 1N4148 diodes;
 - 1 1.8kΩ resistor;
 - a [blue pill board](https://wiki.stm32duino.com/index.php?title=Blue_Pill) featuring a STM32F103C8 microcontroller (20KiB RAM, 64 KiB flash, ARM Cortex M3 @72MHz);
 - a micro USB cable
 - a [3D printed case](cad/)
 
You can find everything on [Aliexpress](https://my.aliexpress.com/wishlist/wish_list_product_list.htm?currentGroupId=100000010426396) for about $50 without the case, soldering iron and multimeter.

## Printing the case

You can directly print the [case](cad/case.stl) and the [back](cad/back.stl). You'll need a printed that can print a 250mm wide piece. Else, you can try the 2 part design but be aware that It's not tested.

If you want to change the size of the grid, you can edit the [source file](cad/case.scad). The number of row and columns are at the begining of the file. Just change that to whatever you want (at least 3 rows and 1 columns). With make and openscad installed, you can just type `make` in the `cad/` directory to regenerate the STL files.

Not support is needed. I print with 20% infill and 0.2mm layers.

## Compiling and flashing

For compiling and flashing, please refer to [the blue pill quickstart](https://github.com/TeXitoi/blue-pill-quickstart/blob/master/README.md).

Basically:

```shell
curl https://sh.rustup.rs -sSf | sh
rustup target add thumbv7m-none-eabi
sudo apt-get install gdb-arm-none-eabi openocd
cd keyberon
# connect ST-Link v2 to the blue pill and the computer
# openocd in another terminal
cargo run --release
```

Now, If you connect the blue pill board to a computer using the micro USB port, the computer should detect a keyboard. You can test it by pushing the caps lock key on your keyboard, the green led of the blue pill should light up. You can also simulate a button press by connecting PA7 and PA8, your computer should register a space key press.

As the blue pill [doesn't respect the USB specifications](https://wiki.stm32duino.com/index.php?title=Blue_Pill#Hardware_installation), you need to fix it. Even if that's working on your computer, you'll want that your keyboard works everywhere. A 1.8kΩ resistor between PA12 and 3.3V can do the job.

## Building the keyboard

TODO
