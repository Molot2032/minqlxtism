minqlxtended
======
minqlxtended is a further extension to [MinoMino's](https://github.com/MinoMino) [minqlx](https://github.com/MinoMino/minqlx) modification to the Quake Live Dedicated Server.

minqlxtended is tested on the latest LTS revision of Ubuntu Server. 

minqlxtended powers [The Purgery](https://thepurgery.com).

Installation
============
- Ensure system is completely up-to-date:
```
sudo apt update
sudo apt upgrade -y
```

- Install Python 3:
```
sudo apt-get -y install python3 python3-dev python3-pip
```

- Install Redis, Git and build-essential:
```
sudo apt-get -y install redis-server git build-essential
```

- Clone this repository and compile minqlxtended
```
git clone https://github.com/tjone270/minqlxtended.git
cd minqlxtended
make
```

- Copy everything from `minqlxtended/bin` into your Quake Live Dedicated Server's installation folder (not the baseq3 folder, but it's parent.):

- Clone the plugins repository and get/build Python dependencies. Assuming you're in the directory with all the server files (where you extracted the above files) do:
```
git clone https://github.com/tjone270/minqlxtended-plugins.git
python3 -m pip install -r minqlxtended-plugins/requirements.txt
```

**IMPORTANT**: Don't be running the above using `sudo` or within the system context. Follow best practices and install these under the service account user you'll be executing `qzeroded` with, these packages will be local to that user and won't interfere with Ubuntu's built-in Python packages.

- Redis should work right off the bat, but you might want to edit the config and make it use UNIX sockets instead for the sake of speed. minqlxtended is configured through CVARs, just like you would configure `qzeroded`. This means you can use `server.cfg` or by passing the CVARs as command line arguments with `+set`. All the CVARs have default values, except for `qlx_owner`, which must contain your SteamID64. Note that the listed owner operates outside of the built-in permission system and can execute any command (and even raw Python).

- You're almost there. Now simply edit the scripts you use to launch the server, but make it point to `run_server_x64_minqlxtended.sh` instead of `run_server_x64.sh`.

Configuration
=============
minqlxtended is configured using CVARs, like you would configure `qzeroded`. All minqlxtended CVARs should be prefixed with `qlx_`. The following CVARs are referenced by the core directly during initialisation and ongoing operation. 
For plugin configuration see the [plugins repository](https://github.com/MinoMino/minqlxtended-plugins).

- `qlx_owner`: The SteamID64 of the server owner. This is should be set, otherwise minqlxtended can't tell who the owner is and will refuse to execute admin commands, unless permission levels are pre-defined in the database.
- `qlx_plugins`: A comma-separated list of plugins that should be loaded at launch.
  - Default: `plugin_manager, essentials, motd, permission, ban, silence, clan, names, log, workshop`.
- `qlx_pluginsPath`: The path (either relative or absolute) to the directory with the plugins.
  - Default: `minqlxtended-plugins`
- `qlx_database`: The default database 'driver' to use. You should not change this unless you know what you're doing.
  - Default: `Redis`
- `qlx_commandPrefix`: The prefix used before command names in order to execute them.
  - Default: `!`
- `qlx_redisAddress`: The address to the Redis database. Can be a path if `qlx_redisUnixSocket` is `"1"`.
  - Default: `127.0.0.1`
- `qlx_redisDatabase`: The Redis database number (by default there are 16 available (zero-indexed)).
  - Default: `0`
- `qlx_redisUnixSocket`: A boolean that determines whether or not `qlx_redisAddress` is a path to a UNIX socket.
  - Default: `0`
- `qlx_redisPassword`: The password to the Redis server, if any.
  - Default: None
- `qlx_logs`: The maximum number of logs the server keeps. `"0"` disables retention processing.
  - Default: `5`
- `qlx_logsSize`: The maximum size in bytes of a log before it backs it up and starts on a fresh file. `"0"` removes the size limitation.
  - Default: `5000000` (5 MB)

Usage
=====
Once you've configured the above CVARs and launched the server, you will quickly recognize if for instance your database configuration is wrong, as it will start printing a bunch of errors in the server console when someone connects. If you only see stuff like the following, then you know it's working like it should:
```
[minqlxtended.late_init] INFO: Loading preset plugins...
[minqlxtended.load_plugin] INFO: Loading plugin 'xxx'...
[minqlxtended.load_plugin] INFO: Loading plugin 'yyy'...
[minqlxtended.load_plugin] INFO: Loading plugin 'zzz'...
[minqlxtended.late_init] INFO: Stats listener started on tcp://127.0.0.1:?????.
[minqlxtended.late_init] INFO: We're good to go!
```

To confirm minqlxtended recognizes you as the owner, try connecting to the server and type `!myperm` in chat.
If it tells you that you have permission level 0, the `qlx_owner` CVAR has not been set correctly (must use the SteamID64 number beginning with 765). Otherwise you should be good to go. As the owner, you are allowed to type commands directly into the console instead of having to use chat. You can now go ahead and add other admins too with `!setperm`.

[See here for a full command list.](https://github.com/tjone270/minqlxtended/wiki/Command-List)
