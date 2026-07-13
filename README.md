# mixer_hotkey

Use hotkeys to manage the volume of your applications.

## Get started

Download mixer_hotkey from the `Release` section, setup a `config.toml` file to create your hotkeys (follow the subsection below for more informations) and put it next to `mixer_hotkey.exe`. Then start the program, you can use `1` to list all opened audio session (useful if you need the exact name of the application to create your hotkeys), or `2` to start executing your hotkeys from the `config.toml`.

If you open an application after starting your hotkeys, you have to restart the hotkeys : just enter `q` to quit hotkey execution and restart it by typing `2`.

## Config

mixer_hotkey works thanks to a config file, here is how to use it.

Such a config file uses the `TOML` format, it must be named `config.toml` and places alongside the executable.

A minimal example of `config.toml` looks like this :

```toml
[[bindings]]
app = "firefox.exe"
key = 173
modifier = 4
delta = 0
```

The various fields are :

- `app` is a string containing the name of the executable the binding controls, a more precise path can be provided
- `key` is the keycode of the key triggering the hotkey, a list can be found at [https://cherrytree.at/misc/vk.htm](https://cherrytree.at/misc/vk.htm)
- `modifier` is the code for the modifier triggering the hotkey, a table can be found below
- `delta` is the value (between -100 and 100) corresponding to the modification of the volume when the hotkey is triggered. A value of 0 correspond to a behavior of muting/unmuting.

| Modifier | Code |
| -------- | ---- |
| Alt | 1 |
| Control | 2 |
| Shift | 4 |
| Windows | 8 |

Multiple `[[bindings]]` can be placed in the `config.toml` to create several hotkeys.