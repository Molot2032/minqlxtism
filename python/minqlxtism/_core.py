# minqlxtism - Extends Quake Live's dedicated server with extra functionality and scripting.
# Copyright (C) 2015 Mino <mino@minomino.org>

# This file is part of minqlxtism.

# minqlxtism is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# minqlxtism is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.

# You should have received a copy of the GNU General Public License
# along with minqlxtism. If not, see <http://www.gnu.org/licenses/>.


# Since this isn't the actual module, we define it here and export
# it later so that it can be accessed with minqlxtism.__doc__ by Sphinx.

import minqlxtism
import minqlxtism.database
import collections
import subprocess
import threading
import traceback
import importlib
import datetime
import os.path
import logging
import shlex
import sys
import os

from logging.handlers import RotatingFileHandler

if sys.version_info < (3, 9):
    raise AssertionError("Only Python 3.9 and later is supported by minqlxtism")

# Team number -> string
TEAMS = collections.OrderedDict(enumerate(("free", "red", "blue", "spectator")))

# Game type number -> string
GAMETYPES = collections.OrderedDict(
    [
        (i, gt)
        for i, gt in enumerate(
            (
                "Free for All",
                "Duel",
                "Race",
                "Team Deathmatch",
                "Clan Arena",
                "Capture the Flag",
                "One Flag",
                "Overload",
                "Harvester",
                "Freeze Tag",
                "Domination",
                "Attack and Defend",
                "Red Rover",
            )
        )
        if gt
    ]
)

# Game type number -> short string
GAMETYPES_SHORT = collections.OrderedDict(
    [(i, gt) for i, gt in enumerate(("ffa", "duel", "race", "tdm", "ca", "ctf", "1f", "ol", "har", "ft", "dom", "ad", "rr")) if gt]
)

# Connection states.
CONNECTION_STATES = collections.OrderedDict(enumerate(("free", "zombie", "connected", "primed", "active")))

WEAPONS = collections.OrderedDict(
    [(i, w) for i, w in enumerate(("", "g", "mg", "sg", "gl", "rl", "lg", "rg", "pg", "bfg", "gh", "ng", "pl", "cg", "hmg", "hands")) if w]
)

DEFAULT_PLUGINS = ("plugin_manager", "essentials", "motd", "permission", "ban", "silence", "clan", "names", "log", "workshop")

# ====================================================================
#                               HELPERS
# ====================================================================


def parse_variables(infostring: str):
    """
    Parses the Quake Live info-string format and places keys/values into an ordered dictionary.

    :param infostring: The info-string with variables.
    :type infostring: str
    :returns: dict -- A dictionary with the variables added as key-value pairs.
    """

    res = collections.OrderedDict()
    if not infostring.strip():
        return res

    vars = infostring.lstrip("\\").split("\\")
    try:
        for i in range(0, len(vars), 2):
            res[vars[i]] = vars[i + 1]
    except IndexError:
        # Log and return incomplete dict.
        logger = minqlxtism.get_logger()
        logger.warning("Uneven number of keys and values: {}".format(infostring))

    return res


def stringify_variables(variables: dict | collections.OrderedDict):
    """
    Converts a dictionary of variables to the Quake Live info-string format.

    :param variables: The variables to convert.
    :type variables: dict
    :returns: str -- The info-string with the variables.
    """

    return "".join(["\\" + str(k) + "\\" + str(v) for k, v in variables.items()])


main_logger = None


def get_logger(plugin=None):
    """
    Provides a logger that should be used by your plugin for debugging, info
    and error reporting. It will automatically output to both the server console
    as well as to a file.

    :param plugin: The plugin that is using the logger.
    :type plugin: minqlxtism.Plugin
    :returns: logging.Logger -- The logger in question.
    """
    if plugin:
        return logging.getLogger("minqlxtism." + str(plugin))
    else:
        return logging.getLogger("minqlxtism")


def _configure_logger():
    logger = logging.getLogger("minqlxtism")
    logger.setLevel(logging.DEBUG)

    # File
    file_path = os.path.join(minqlxtism.get_cvar("fs_homepath"), "minqlxtism.log")
    maxlogs = minqlxtism.Plugin.get_cvar("qlx_logs", int)
    maxlogsize = minqlxtism.Plugin.get_cvar("qlx_logsSize", int)
    file_fmt = logging.Formatter("(%(asctime)s) [%(levelname)s @ %(name)s.%(funcName)s] %(message)s", "%H:%M:%S")
    file_handler = RotatingFileHandler(file_path, encoding="utf-8", maxBytes=maxlogsize, backupCount=maxlogs)
    file_handler.setLevel(logging.DEBUG)
    file_handler.setFormatter(file_fmt)
    logger.addHandler(file_handler)
    logger.info("============================= minqlxtism run @ {} =============================".format(datetime.datetime.now()))

    # Console
    console_fmt = logging.Formatter("[%(name)s.%(funcName)s] %(levelname)s: %(message)s", "%H:%M:%S")
    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.INFO)
    console_handler.setFormatter(console_fmt)
    logger.addHandler(console_handler)


def log_exception(plugin=None):
    """
    Logs an exception using :func:`get_logger`. Call this in an except block.

    :param plugin: The plugin that is using the logger.
    :type plugin: minqlxtism.Plugin
    """
    # TODO: Remove plugin arg and make it automatic.
    logger = get_logger(plugin)
    e = traceback.format_exc().rstrip("\n")
    for line in e.split("\n"):
        logger.error(line)


def handle_exception(exc_type, exc_value, exc_traceback):
    """A handler for unhandled exceptions."""
    # TODO: If exception was raised within a plugin, detect it and pass to log_exception()
    logger = get_logger(None)
    e = "".join(traceback.format_exception(exc_type, exc_value, exc_traceback)).rstrip("\n")
    for line in e.split("\n"):
        logger.error(line)


def threading_excepthook(args):
    handle_exception(args.exc_type, args.exc_value, args.exc_traceback)


_init_time = datetime.datetime.now()


def uptime():
    """Returns a :class:`datetime.timedelta` instance of the time since initialized."""
    return datetime.datetime.now() - _init_time


def owner():
    """Returns the SteamID64 of the owner. This is set in the config."""
    try:
        sid = int(minqlxtism.get_cvar("qlx_owner"))
        if sid == -1:
            raise RuntimeError
        return sid
    except:
        logger = minqlxtism.get_logger()
        logger.error("Failed to parse the Owner Steam ID. Make sure it's in SteamID64 format.")


_stats = None


def stats_listener():
    """Returns the :class:`minqlxtism.StatsListener` instance used to listen for stats."""
    return _stats


def set_cvar_once(name, value, flags=0):
    if minqlxtism.get_cvar(name) is None:
        minqlxtism.set_cvar(name, value, flags)
        return True

    return False


def set_cvar_limit_once(name, value, minimum, maximum, flags=0):
    if minqlxtism.get_cvar(name) is None:
        minqlxtism.set_cvar_limit(name, value, minimum, maximum, flags)
        return True

    return False


def set_plugins_version(path):
    args_version = shlex.split("git describe --long --tags --dirty --always")
    args_branch = shlex.split("git rev-parse --abbrev-ref HEAD")

    # We keep environment variables, but remove LD_PRELOAD to avoid a warning the OS might throw.
    env = dict(os.environ)
    del env["LD_PRELOAD"]
    try:
        # Get the version using git describe.
        p = subprocess.Popen(args_version, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=path, env=env)
        p.wait(timeout=1)
        if p.returncode != 0:
            setattr(minqlxtism, "__plugins_version__", "NOT_SET")
            return

        version = p.stdout.read().decode().strip()

        # Get the branch using git rev-parse.
        p = subprocess.Popen(args_branch, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=path, env=env)
        p.wait(timeout=1)
        if p.returncode != 0:
            setattr(minqlxtism, "__plugins_version__", version)
            return

        branch = p.stdout.read().decode().strip()
    except (FileNotFoundError, subprocess.TimeoutExpired):
        setattr(minqlxtism, "__plugins_version__", "NOT_SET")
        return

    setattr(minqlxtism, "__plugins_version__", "{}-{}".format(version, branch))


def set_map_subtitles():
    # We save the actual values before setting them so that we can retrieve them in Game.
    setattr(minqlxtism, "_map_title", minqlxtism.get_configstring(3))
    setattr(minqlxtism, "_map_subtitle1", minqlxtism.get_configstring(678))
    setattr(minqlxtism, "_map_subtitle2", minqlxtism.get_configstring(679))

    cs = minqlxtism.get_configstring(678)
    if cs:
        cs += " - "
    minqlxtism.set_configstring(
        678, cs + "Running minqlxtism ^6{}^7 with plugins ^6{}^7.".format(minqlxtism.__version__, minqlxtism.__plugins_version__)
    )
    cs = minqlxtism.get_configstring(679)
    if cs:
        cs += " - "
    minqlxtism.set_configstring(679, cs + "Check ^6http://github.com/tjone270/minqlxtism^7 for more details.")


# ====================================================================
#                              DECORATORS
# ====================================================================


def next_frame(func):
    def f(*args, **kwargs):
        minqlxtism.next_frame_tasks.append((func, args, kwargs))

    return f


def delay(time):
    """Delay a function call a certain amount of time.

    .. note::
        It cannot guarantee you that it will be called right as the timer
        expires, but unless some plugin is for some reason blocking, then
        you can expect it to be called practically as soon as it expires.

    :param func: The function to be called.
    :type func: callable
    :param time: The number of seconds before the function should be called.
    :type time: int

    """

    def wrap(func):
        def f(*args, **kwargs):
            minqlxtism.frame_tasks.enter(time, 0, func, args, kwargs)

        return f

    return wrap


_thread_count = 0
_thread_name = "minqlxtismthread"


def thread(func, force=False):
    """Starts a thread with the function passed as its target. If a function decorated
    with this is called within a function also decorated, it will **not** create a second
    thread unless told to do so with the *force* keyword.

    :param func: The function to be ran in a thread.
    :type func: callable
    :param force: Force it to create a new thread even if already in one created by this decorator.
    :type force: bool
    :returns: threading.Thread

    """

    def f(*args, **kwargs):
        if not force and threading.current_thread().name.endswith(_thread_name):
            func(*args, **kwargs)
        else:
            global _thread_count
            name = func.__name__ + "-{}-{}".format(str(_thread_count), _thread_name)
            t = threading.Thread(target=func, name=name, args=args, kwargs=kwargs, daemon=True)
            t.start()
            _thread_count += 1

            return t

    return f


# ====================================================================
#                       CONFIG AND PLUGIN LOADING
# ====================================================================

# We need to keep track of module instances for use with importlib.reload.
_modules = {}


class PluginLoadError(Exception):
    pass


class PluginUnloadError(Exception):
    pass


def load_preset_plugins():
    plugins_temp = []
    for p in minqlxtism.Plugin.get_cvar("qlx_plugins", list):
        if p == "DEFAULT":
            plugins_temp += list(DEFAULT_PLUGINS)
        else:
            plugins_temp.append(p)

    plugins = []
    for p in plugins_temp:
        if p not in plugins:
            plugins.append(p)

    plugins_path = os.path.abspath(minqlxtism.get_cvar("qlx_pluginsPath"))
    plugins_dir = os.path.basename(plugins_path)

    if os.path.isdir(plugins_path):
        plugins = [p for p in plugins if "{}.{}".format(plugins_dir, p)]
        for p in plugins:
            load_plugin(p)
    else:
        raise (PluginLoadError("Cannot find the plugins directory '{}'.".format(os.path.abspath(plugins_path))))


def load_plugin(plugin):
    logger = get_logger(None)
    logger.info("Loading plugin '{}'...".format(plugin))
    plugins = minqlxtism.Plugin._loaded_plugins
    plugins_path = os.path.abspath(minqlxtism.get_cvar("qlx_pluginsPath"))
    plugins_dir = os.path.basename(plugins_path)

    if not os.path.isfile(os.path.join(plugins_path, plugin + ".py")):
        raise PluginLoadError("No such plugin exists.")
    elif plugin in plugins:
        return reload_plugin(plugin)
    try:
        module = importlib.import_module("{}.{}".format(plugins_dir, plugin))
        # We add the module regardless of whether it fails or not, otherwise we can't reload later.
        global _modules
        _modules[plugin] = module

        if not hasattr(module, plugin):
            raise (PluginLoadError("The plugin needs to have a class with the exact name as the file, minus the .py."))

        plugin_class = getattr(module, plugin)
        if issubclass(plugin_class, minqlxtism.Plugin):
            plugins[plugin] = plugin_class()
        else:
            raise (PluginLoadError("Attempted to load a plugin that is not a subclass of 'minqlxtism.Plugin'."))
    except:
        log_exception(plugin)
        raise


def unload_plugin(plugin):
    logger = get_logger(None)
    logger.info("Unloading plugin '{}'...".format(plugin))
    plugins = minqlxtism.Plugin._loaded_plugins
    if plugin in plugins:
        try:
            minqlxtism.EVENT_DISPATCHERS["unload"].dispatch(plugin)

            # Unhook its hooks.
            for hook in plugins[plugin].hooks:
                plugins[plugin].remove_hook(*hook)

            # Unregister commands.
            for cmd in plugins[plugin].commands:
                plugins[plugin].remove_command(cmd.name, cmd.handler)

            del plugins[plugin]
        except:
            log_exception(plugin)
            raise
    else:
        raise (PluginUnloadError("Attempted to unload a plugin that is not loaded."))


def reload_plugin(plugin):
    try:
        unload_plugin(plugin)
    except PluginUnloadError:
        pass

    try:
        global _modules
        if plugin in _modules:  # Unloaded previously?
            importlib.reload(_modules[plugin])
        load_plugin(plugin)
    except:
        log_exception(plugin)
        raise


def initialize_cvars():
    # Core
    minqlxtism.set_cvar_once("qlx_owner", "-1")
    minqlxtism.set_cvar_once("qlx_plugins", ", ".join(DEFAULT_PLUGINS))
    minqlxtism.set_cvar_once("qlx_pluginsPath", "minqlxtism-plugins")
    minqlxtism.set_cvar_once("qlx_database", "Redis")
    minqlxtism.set_cvar_once("qlx_commandPrefix", "!")
    minqlxtism.set_cvar_once("qlx_logs", "2")
    minqlxtism.set_cvar_once("qlx_logsSize", str(3 * 10**6))  # 3 MB
    # Redis
    minqlxtism.set_cvar_once("qlx_redisAddress", "127.0.0.1")
    minqlxtism.set_cvar_once("qlx_redisDatabase", "0")
    minqlxtism.set_cvar_once("qlx_redisProtocol", "3")
    minqlxtism.set_cvar_once("qlx_redisUnixSocket", "0")
    minqlxtism.set_cvar_once("qlx_redisPassword", "")


# ====================================================================
#                                 MAIN
# ====================================================================


def initialize():
    minqlxtism.register_handlers()


def late_init():
    """Initialization that needs to be called after QLDS has finished
    its own initialization.

    """
    minqlxtism.initialize_cvars()

    # Set the default database plugins should use.
    # TODO: Make Plugin.database setting generic.
    if minqlxtism.get_cvar("qlx_database").lower() == "redis":
        minqlxtism.Plugin.database = minqlxtism.database.Redis

    # Get the plugins path and set minqlxtism.__plugins_version__.
    plugins_path = os.path.abspath(minqlxtism.get_cvar("qlx_pluginsPath"))
    set_plugins_version(plugins_path)

    # Initialize the logger now that we have fs_basepath.
    _configure_logger()
    logger = get_logger()
    # Set our own exception handler so that we can log them if unhandled.
    sys.excepthook = handle_exception

    if sys.version_info >= (3, 8):
        threading.excepthook = threading_excepthook

    # Add the plugins path to PATH so that we can load plugins later.
    sys.path.append(os.path.dirname(plugins_path))

    logger.info("Loading preset plugins...")
    load_preset_plugins()

    if bool(int(minqlxtism.get_cvar("zmq_stats_enable"))):
        global _stats
        _stats = minqlxtism.StatsListener()
        logger.info("Stats listener started on {}.".format(_stats.address))
        # Start polling. Not blocking due to decorator magic. Aw yeah.
        _stats.keep_receiving()

    logger.info("We're good to go!")
