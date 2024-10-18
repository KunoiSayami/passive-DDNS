# passive-DDNS

Design for [CloudFlare](https://cloudflare.com) , It use in has public IP home network to auto change `A` record which domain use cloudflare ns.

## Notice

This project only support cloudflare NS and custom upstream (update your IP by post value).

Python version is deprecated after `b690919c83f7fb879c2e34db8cb7e87262d0f565` commit.

## Usage

Copy `config.toml.default` to `config.ini`, parse your `token`, `domain`, in configure file. Then, using this command to run program:

```bash
cargo run --release
```

### With Systemd

You should build this project first:

```shell
cargo build --release
```

Then, put `data/passive-ddns.service` to `/etc/systemd/system` folder.

After exec daemon reload command:

```shell
sudo systemctl daemon-reload
```

You should can see the status of `passive-ddns.service`.

Don't forget enable them by:

```shell
sudo systemctl enable --now passive-ddns.service
```

## License

[![](https://www.gnu.org/graphics/agplv3-155x51.png "AGPL v3 logo")](https://www.gnu.org/licenses/agpl-3.0.txt)

Copyright (C) 2018-2024 KunoiSayami

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
