---
tags:
  - mouse
---
# `alternate_buffer_wheel_scroll_speed = 3`

{{since('20210203-095643-70a364eb')}}

Normally the vertical mouse wheel will scroll the terminal viewport
so that different sections of the scrollback are visible.

When an application activates the *Alternate Screen Buffer* (this is
common for "full screen" terminal programs such as pagers and editors),
the alternate screen doesn't have a scrollback.

In this mode, if the application hasn't enabled mouse reporting, wezterm will
generate Arrow Up/Down key events when the vertical mouse wheel is scrolled.

The `alternate_buffer_wheel_scroll_speed` specifies how many arrow key presses
are generated by a single scroll wheel "tick".

The default for this value is `3`, which means that a single wheel up tick will
appear to the application as though the user pressed arrow up three times in
quick succession.

In versions of wezterm prior to this configuration option being available, the
behavior was the same except that the effective value of this option was always
`1`.
