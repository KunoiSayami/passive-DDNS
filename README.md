# passive-DDNS

Design for [hosker](https://zhujike.com), It use in has public IP home network to auto change A record which domain use hosker ns.

## Usage

In principle, need python 2.7.x interpreter. And `requests` library must avaliable.

If you use it in first time, you need run `git submodule update --init` to get libpy file.

Copy `config.ini.default` to `config.ini`, parse your `email`, `token`, `header_domain`, in configure file. Then, using this command to run program:

```bash
python main.py
```

## License

[![](https://www.gnu.org/graphics/agplv3-155x51.png)](https://www.gnu.org/licenses/agpl-3.0.txt)

Copyright (C) 2018 Too-Naive

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
