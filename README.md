# middle-button-doubleclick

An ultra-lightweight Linux daemon written in Rust that intercepts your physical mouse's middle-click button and maps it natively to a double left-click. 

Unlike traditional macro utilities, this tool safely operates at the kernel event layer (`evdev`/`uinput`), making it completely agnostic to your display server (it functions flawlessly under both **Wayland** and **X11** sessions) and robust against aggressive desktop compositors like KDE Plasma's KWin.

## Features
* **Zero Runtime Overhead:** Compiled machine binary with near-zero CPU footprint and tiny (~3MB) RAM usage.
* **Auto-Hardware Detection:** Automatically hooks your mouse device on startup, even if you change USB ports or reboot.
* **Resilient Architecture:** Spawns multi-threaded sub-device listeners to capture nested input streams, safely absorbing native clipboard paste actions.

## Prerequisites (Ubuntu/Kubuntu/Debian)
Before building, you will need the Rust toolchain and system hardware header libraries:

```bash
sudo apt update && sudo apt install -y cargo pkg-config libudev-dev build-essential
```

## Installation & Deployment

1. Clone this repository to your computer.
2. Run the provided install script from the project root:

```bash
./install.sh
```

The script will automatically compile an optimized release binary, safely configure a persistent background `systemd` system unit, and launch the daemon. It will configure itself to run automatically on every system boot without ever prompting you for a password.

## Managing the Daemon
You can check the status or inspect live hardware intercept signals via standard system logging:

* **View live logs:** `journalctl -u middle-button-doubleclick.service -f`
* **Stop daemon temporarily:** `sudo systemctl stop middle-button-doubleclick.service`
* **Restart daemon:** `sudo systemctl restart middle-button-doubleclick.service`

## License
This project is open-source software licensed under the **GNU General Public License (GPL)**.
