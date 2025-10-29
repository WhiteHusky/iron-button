# Iron Button

```
Simple program that uses XDG portals to bind global shortcuts to user-defined actions going beyond simple key-down.

Usage: iron-button [OPTIONS]

Options:
  -v, --verbose             whether to be verbose
      --show-portal-config  reshow the XDG portal configuration dialog
      --config <CONFIG>     override XDG_CONFIG_HOME/iron-button/config.yml with a different path
  -h, --help                Print help
  -V, --version             Print version
```

Currently a prototype. If ran inside a terminal emulator, it may assume its identity. Otherwise it might be listed as `ashpd_AABBCC` if started as a systemd unit or other circumstances.

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