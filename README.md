# passive-DDNS
Design for hosker, It use in has public IP home network to auto change A record which domain use hosker ns.

## Usage

In principle, need python 2.7.x interpreter. And `requests` library must avaliable.

If you use it in first time, you need run `git submodule update --init` to get libpy file.

Copy `config.ini.default` to `config.ini`, parse your `email`, `token`, `header`, `domain`, in configure file. Then, using this command to run program:

```bash
python main.py
```

## LICENSE
AGPL v3
