# YWpM - YoRHa Wallpaper Manager
YoRHa Wallpaper Manager is a modern, user‑friendly console tool that helps you manage all your wallpapers on Linux. Please note that YWpM only determines when and which wallpapers to render, and does not render wallpapers on its own. Instead, YWpM uses a backend. Currently, only mpvpaper is supported as a backend.

## Build

### 1. Dependencies
* cargo
* mpvpaper ### 2. Compiling and building the package
#### 2.1. Building a DEB package:
``` bash
cargo install cargo-deb
cargo deb
sudo apt-get install ./target/debian/*.deb
```

#### 2.2. Building an RPM package:
``` bash
cargo install cargo-generate-rpm
cargo generate-rpm
Installation for **Fedora / RHEL / CentOS**:
``` bash
sudo dnf install ./target/generate-rpm/*.rpm
```

Installation for **Alt Linux**:
``` bash
sudo apt-get install ./target/generate-rpm/*.rpm
```

#### 2.3. Compilation without package building
``` bash
cargo build --release
```

### 3. Service startup
The service does not start automatically (due to systemd specifics). After installation, you need to run:
``` bash
systemctl --user daemon-reload
systemctl --user start ywpmd.service
```

## Usage
The `ywpmd` service is **not** added to systemd autostarting in principle - `systemctl enable` will not work. This decision was made after several unsuccessful attempts to get systemd to start it at the right time. Just add an autostart entry for your system: `systemctl --user start ywpmd.service`. If a black screen appears after loading, try adding `sleep 1`. This delay usually helps. If it doesn’t help, check `systemctl --user status ywpmd.service`.

 After the service is successfully launched, it will display a blank screen. It is ok. The service doesn’t yet know which wallpapers need to be displayed. Add them:
``` bash
ywpm add <path>
```

**\<path\>** - the path to the folder or file. For a folder, it adds all wallpapers. By default, it’s not recursive. To specify the nesting level, use:
``` bash
ywpm add -r 65535 <path>
```

`65535` - the maximum nesting level.

After that, you can set random wallpapers:
``` bash
ywpm random
```

**Examples:**
* `ywpm set /path/to/file.png` - set the wallpaper for the current session (or until the next any change of period)
* `ywpm reset` - reset the wallpaper. It will show a gray/black screen
* `ywpm get` or `ywpm get -v` - view the current wallpaper. The **-v** flag shows additional information
* `ywpm list` or `ywpm list -v` - view all wallpapers
* `ywpm remove /path/to/file.png` - remove a wallpaper from the program. **Does not** delete the file/folder from the disk
* `ywpm clear` - remove all wallpapers from the program. **Does not** delete anything from the disk
* `ywpm find-non-fitting` - Find all images and videos whose aspect ratio differs from the monitor and for which --mode is not specified. For video, it also checks for pixel‑by‑pixel resolution matching, since real‑time video scaling is an expensive operation.

## Advanced usage. Options
### -m, --mode
Wallpaper display mode. You can set vertical alignment (top/center/bottom), horizontal alignment (left/center/right), and stretching mode:

* `cover` - Crops the edges, preserves proportions
* `contain` - Leaves empty space at the edges, preserves proportions
* `stretch` - Does not preserve aspect ratio, stretching the image

**Default:** `center cover`

**Examples:**

* `ywpm add /path -m 'cover right'` - The image will be shifted to the right and cropped if it is wider than the screen
* `ywpm add /path -m 'top contain'` - The image will be shifted upward without cropping if it is wider than the screen

### -r, --recursive-level
Level of recursive file search in the folder. Range: `1..65535`

**Default:** `1`

## Advanced usage. Time periods
A wallpaper or a group of wallpapers can be set for a specific time period. If several wallpapers and/or groups are set at some point, random wallpapers are selected from all installed ones with equal probability. To set a period, there are 3 options: **--since**, **--until**, and **--duration**. The **--until** and **--duration** options cannot be specified simultaneously. If the **--since** option is specified, then one of the **--until** or **--duration** options must be specified — the program does not allow you to set the wallpaper indefinitely. If you still want to, you can set it for 1,000 years in advance: **--duration=1,000y**

### -s, --since
The start of the period with minute‑level precision. Does not take the time zone into account. You can specify the integer and fractional parts of seconds and the time zone. They will be ignored. This decision was made to ensure compatibility with the most common time formats.

**Examples:**

* `14:00` - today at 14:00
* `2026-06-11 12:00`
* `27 jul 2026 20:08:32 +0000` - turns into `2026-07-27 20:08`
* `now` - current time
* `tomorrow` - current time plus one day

**Default:** `now`

### -u, --until
End of the period with minute precision. The format is the same as for **--since**.

**Default:** \<not specified\>

### -d, --duration
Duration of the period.

**Examples:**

* `30m`, `30 minutes` - half an hour
* `12h`, `12 hours` - 12 hours
* `2w` - 2 weeks
* `5 month` - 5 months. Months are counted as 30 days
* `1 y` - 5 years. * `20:30` - The format here is the same as for **--since** (except for `now` and `tomorrow`)

## Advanced usage. Groups
A group in ywpm is, in fact, a group of files and/or folders. You can do almost everything with groups that you can do with individual folders/files - set them via `set` (in which case a random file from the group is selected), schedule them for a certain period. However, the **--mode** and **--recursive-level** options are not available for groups. Here are all the main actions that can be performed on groups:

* `group-list` - Show a list of all groups
* `get-group` - Show information and composition of a group
* `new-group` - Create a new group
* `set-group` - Set random wallpapers from a group
* `add-to-group` - Add files/folders to a group
* `remove-from-group` - Remove files/folders from a group (not from the disk)
* `clear-group` - Clear the group
* `remove-group` - Remove the group
