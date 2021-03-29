# v0.2.0

* New Keyboard::leds_mut function for getting underlying leds object.
* Made Layout::current_layer public for getting current active layer.

Breaking changes:
* Update to generic_array 0.14, that is exposed in matrix. The update
  should be transparent.
* `Action::HoldTap` now takes a configuration for different behaviors.
* `Action::HoldTap` now takes the `tap_hold_interval` field. Not
  implemented yet.
* `Action` is now generic, for the `Action::Custom(T)` variant,
  allowing custom action to be handled outside of keyberon. This
  functionality can be used to drive non keyboard actions, as reset
  the microcontroller, drive leds (for backlight or underglow for
  example), manage a mouse emulation, or any other ideas you can
  have. As there is a default value for the type parameter, the update
  should be transparent.
* Rename MeidaCoffee in MediaCoffee to fix typo.

# v0.1.1

*  HidClass::control_xxx: check interface number [#26](https://github.com/TeXitoi/keyberon/pull/26)

# v0.1.0

First published version.
