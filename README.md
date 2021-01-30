# passive-DDNS

Design for [CloudFlare](https://cloudflare.com) and [Hostker](https://zhujike.com), It use in has public IP home network to auto change `A` record which domain use cloudflare/hostker ns.

## Notice

Hostker library is not longer maintenance after `0def47c655c4a8db3359c1fc34f280ac26431962`.

This project is in RIIR progress, Python version will deprecated after RIIR finished.

## Usage

In principle, need Python 3.7.x interpreter. And `requests`, `bs4` library must available.

Copy `config.ini.default` to `config.ini`, parse your `email` (hostker), `token`, `header_domain`, in configure file. Then, using this command to run program:

```bash
python3 pddns.py
```

### Reload

Send `SIGUSR1` signal to main process for skip waiting.

## License

[![](https://www.gnu.org/graphics/agplv3-155x51.png)](https://www.gnu.org/licenses/agpl-3.0.txt)

Copyright (C) 2018-2020 KunoiSayami

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
