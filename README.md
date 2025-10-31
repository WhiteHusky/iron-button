# Iron Button

```
Simple program that uses XDG portals to bind global shortcuts to user-defined actions going beyond simple key-down.

Usage: iron-button [OPTIONS]

Options:
  -v, --verbose             Whether to be verbose
      --show-portal-config  Reshow the XDG portal configuration dialog
      --config <CONFIG>     Override XDG_CONFIG_HOME/iron-button/config.yml with a different path
  -h, --help                Print help
  -V, --version             Print version
```

Currently a prototype. If ran inside a terminal emulator, even if inside a larger program like Codium or Dolphin, it may assume its identity and list shortcuts under it in your system settings. Otherwise it might be listed as `ashpd_AABBCC` if started as a systemd unit or other circumstances.

You can also edit the config and send a SIGHUP to the process to reload the config without restarting the program.

## Config Sample

```yml
binds:
  test bind:
    description: Demonstration bind
    suggest: LOGO+ALT+Return
    on_down:
      !Run
        program: echo
        arguments:
          - "Down!"
    on_up:
      !Run
        program: echo
        arguments:
          - "Up!"
```