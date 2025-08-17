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

import minqlxtism
import collections
import sched
import re

# ====================================================================
#                        REGULAR EXPRESSIONS
# ====================================================================

_re_say = re.compile(r"^say +\"?(?P<msg>.+)\"?$", flags=re.IGNORECASE)
_re_say_team = re.compile(r"^say_team +\"?(?P<msg>.+)\"?$", flags=re.IGNORECASE)
_re_callvote = re.compile(r"^(?:cv|callvote) +(?P<cmd>[^ ]+)(?: \"?(?P<args>.+?)\"?)?$", flags=re.IGNORECASE)
_re_vote = re.compile(r"^vote +(?P<arg>.)", flags=re.IGNORECASE)
_re_team = re.compile(r"^team +(?P<arg>.)", flags=re.IGNORECASE)
_re_vote_ended = re.compile(r"^print \"Vote (?P<result>passed|failed).\n\"$")
_re_userinfo = re.compile(r"^userinfo \"(?P<vars>.+)\"$")

# ====================================================================
#                         LOW-LEVEL HANDLERS
#        These are all called by the C code, not within Python.
# ====================================================================

def handle_rcon(cmd):
    """Console commands that are to be processed as regular pyminqlxtism
    commands as if the owner executes it. This allows the owner to
    interact with the Python part of minqlxtism without having to connect.

    """
    try:
        minqlxtism.COMMANDS.handle_input(minqlxtism.RconDummyPlayer(), cmd, minqlxtism.CONSOLE_CHANNEL)
    except:
        minqlxtism.log_exception()
        return True

def handle_client_command(client_id, cmd):
    """Client commands are commands such as "say", "say_team", "scores",
    "disconnect" and so on. This function parses those and passes it
    on to the event dispatcher.

    :param client_id: The client identifier.
    :type client_id: int
    :param cmd: The command being ran by the client.
    :type cmd: str

    """
    try:
        # Dispatch the "client_command" event before further processing.
        player = minqlxtism.Player(client_id)
        retval = minqlxtism.EVENT_DISPATCHERS["client_command"].dispatch(player, cmd)
        if retval is False:
            return False
        elif isinstance(retval, str):
            # Allow plugins to modify the command before passing it on.
            cmd = retval

        res = _re_say.match(cmd)
        if res:
            msg = res.group("msg").replace("\"", "")
            channel = minqlxtism.CHAT_CHANNEL
            if minqlxtism.EVENT_DISPATCHERS["chat"].dispatch(player, msg, channel) is False:
                return False
            return cmd

        res = _re_say_team.match(cmd)
        if res:
            msg = res.group("msg").replace("\"", "")
            if player.team == "free": # I haven't tried this, but I don't think it's even possible.
                channel = minqlxtism.FREE_CHAT_CHANNEL
            elif player.team == "red":
                channel = minqlxtism.RED_TEAM_CHAT_CHANNEL
            elif player.team == "blue":
                channel = minqlxtism.BLUE_TEAM_CHAT_CHANNEL
            else:
                channel = minqlxtism.SPECTATOR_CHAT_CHANNEL
            if minqlxtism.EVENT_DISPATCHERS["chat"].dispatch(player, msg, channel) is False:
                return False
            return cmd

        res = _re_callvote.match(cmd)
        if res and not minqlxtism.Plugin.is_vote_active():
            vote = res.group("cmd")
            args = res.group("args") if res.group("args") else ""
            # Set the caller for vote_started in case the vote goes through.
            minqlxtism.EVENT_DISPATCHERS["vote_started"].caller(player)
            if minqlxtism.EVENT_DISPATCHERS["vote_called"].dispatch(player, vote, args) is False:
                return False
            return cmd

        res = _re_vote.match(cmd)
        if res and minqlxtism.Plugin.is_vote_active():
            arg = res.group("arg").lower()
            if arg == "y" or arg == "1":
                if minqlxtism.EVENT_DISPATCHERS["vote"].dispatch(player, True) is False:
                    return False
            elif arg == "n" or arg == "2":
                if minqlxtism.EVENT_DISPATCHERS["vote"].dispatch(player, False) is False:
                    return False
            return cmd

        res = _re_team.match(cmd)
        if res:
            arg = res.group("arg").lower()
            target_team = ""
            if arg == player.team[0]:
                # Don't trigger if player is joining the same team.
                return cmd
            elif arg == "f":
                target_team = "free"
            elif arg == "r":
                target_team = "red"
            elif arg == "b":
                target_team = "blue"
            elif arg == "s":
                target_team = "spectator"
            elif arg == "a":
                target_team = "any"

            if target_team:
                if minqlxtism.EVENT_DISPATCHERS["team_switch_attempt"].dispatch(player, player.team, target_team) is False:
                    return False
            return cmd

        res = _re_userinfo.match(cmd)
        if res:
            new_info = minqlxtism.parse_variables(res.group("vars"))
            old_info = player.cvars
            changed = {}

            for key in new_info:
                if key not in old_info or (key in old_info and new_info[key] != old_info[key]):
                    changed[key] = new_info[key]

            if changed:
                ret = minqlxtism.EVENT_DISPATCHERS["userinfo"].dispatch(player, changed)
                if ret is False:
                    return False
                elif isinstance(ret, dict):
                    for key in ret:
                        new_info[key] = ret[key]
                    cmd = "userinfo \"{}\"".format("".join(["\\{}\\{}".format(key, new_info[key]) for key in new_info]))

        return cmd
    except:
        minqlxtism.log_exception()
        return True

def handle_server_command(client_id, cmd):
    try:
        # Dispatch the "server_command" event before further processing.
        try:
            player = minqlxtism.Player(client_id) if client_id >= 0 else None
        except minqlxtism.NonexistentPlayerError:
            return True

        retval = minqlxtism.EVENT_DISPATCHERS["server_command"].dispatch(player, cmd)
        if retval is False:
            return False
        elif isinstance(retval, str):
            cmd = retval

        res = _re_vote_ended.match(cmd)
        if res:
            if res.group("result") == "passed":
                minqlxtism.EVENT_DISPATCHERS["vote_ended"].dispatch(True)
            else:
                minqlxtism.EVENT_DISPATCHERS["vote_ended"].dispatch(False)

        return cmd
    except:
        minqlxtism.log_exception()
        return True

# Executing tasks right before a frame, by the main thread, will often be desirable to avoid
# weird behavior if you were to use threading. This list will act as a task queue.
# Tasks can be added by simply adding the @minqlxtism.next_frame decorator to functions.
frame_tasks = sched.scheduler()
next_frame_tasks = collections.deque()

def handle_frame():
    """This will be called every frame. To allow threads to call stuff from the
    main thread, tasks can be scheduled using the :func:`minqlxtism.next_frame` decorator
    and have it be executed here.

    """

    while True:
        # This will run all tasks that are currently scheduled.
        # If one of the tasks throw an exception, it'll log it
        # and continue execution of the next tasks if any.
        try:
            frame_tasks.run(blocking=False)
            break
        except:
            minqlxtism.log_exception()
            continue
    try:
        minqlxtism.EVENT_DISPATCHERS["frame"].dispatch()
    except:
        minqlxtism.log_exception()
        return True

    try:
        while True:
            func, args, kwargs = next_frame_tasks.popleft()
            frame_tasks.enter(0, 0, func, args, kwargs)
    except IndexError:
        pass


_zmq_warning_issued = False
_first_game = True
_ad_round_number = 0

def handle_new_game(is_restart):
    # This is called early in the launch process, so it's a good place to initialize
    # minqlxtism stuff that needs QLDS to be initialized.
    global _first_game
    if _first_game:
        minqlxtism.late_init()
        _first_game = False

        # A good place to warn the owner if ZMQ stats are disabled.
        global _zmq_warning_issued
        if not bool(int(minqlxtism.get_cvar("zmq_stats_enable"))) and not _zmq_warning_issued:
            logger = minqlxtism.get_logger()
            logger.warning("Some events will not work because ZMQ stats is not enabled. "
                "Launch the server with \"zmq_stats_enable 1\"")
            _zmq_warning_issued = True

    minqlxtism.set_map_subtitles()

    if not is_restart:
        try:
            minqlxtism.EVENT_DISPATCHERS["map"].dispatch(
                minqlxtism.get_cvar("mapname"),
                minqlxtism.get_cvar("g_factory"))
        except:
            minqlxtism.log_exception()
            return True

    try:
        minqlxtism.EVENT_DISPATCHERS["new_game"].dispatch()
    except:
        minqlxtism.log_exception()
        return True

def handle_set_configstring(index, value):
    """Called whenever the server tries to set a configstring. Can return
    False to stop the event.

    """
    global _ad_round_number

    try:
        res = minqlxtism.EVENT_DISPATCHERS["set_configstring"].dispatch(index, value)
        if res is False:
            return False
        elif isinstance(res, str):
            value = res

        # VOTES
        if index == 9 and value:
            cmd = value.split()
            vote = cmd[0] if cmd else ""
            args = " ".join(cmd[1:]) if len(cmd) > 1 else ""
            minqlxtism.EVENT_DISPATCHERS["vote_started"].dispatch(vote, args)
            return
        # GAME STATE CHANGES
        elif index == 0:
            old_cs = minqlxtism.parse_variables(minqlxtism.get_configstring(index))
            if not old_cs:
                return

            new_cs = minqlxtism.parse_variables(value)
            old_state = old_cs["g_gameState"]
            new_state = new_cs["g_gameState"]
            if old_state != new_state:
                if old_state == "PRE_GAME" and new_state == "IN_PROGRESS":
                    pass
                elif old_state == "PRE_GAME" and new_state == "COUNT_DOWN":
                    _ad_round_number = 1
                    minqlxtism.EVENT_DISPATCHERS["game_countdown"].dispatch()
                elif old_state == "COUNT_DOWN" and new_state == "IN_PROGRESS":
                    pass
                    #minqlxtism.EVENT_DISPATCHERS["game_start"].dispatch()
                elif old_state == "IN_PROGRESS" and new_state == "PRE_GAME":
                    pass
                elif old_state == "COUNT_DOWN" and new_state == "PRE_GAME":
                    pass
                else:
                    logger = minqlxtism.get_logger()
                    logger.warning("UNKNOWN GAME STATES: {} - {}".format(old_state, new_state))
        # ROUND COUNTDOWN AND START
        elif index == 661:
            cvars = minqlxtism.parse_variables(value)
            if cvars:
                if "turn" in cvars:
                    # it is A&D
                    if int(cvars["state"]) == 0:
                        return
                    # round cvar appears only on round countdown
                    # and first round is 0, not 1
                    try:
                        round_number = int(cvars["round"]) * 2 + 1 + int(cvars["turn"])
                        _ad_round_number = round_number
                    except KeyError:
                        round_number = _ad_round_number
                else:
                    # it is CA
                    round_number = int(cvars["round"])

                if round_number and "time" in cvars:
                    minqlxtism.EVENT_DISPATCHERS["round_countdown"].dispatch(round_number)
                    return
                elif round_number:
                    minqlxtism.EVENT_DISPATCHERS["round_start"].dispatch(round_number)
                    return

        return res
    except:
        minqlxtism.log_exception()
        return True

def handle_player_connect(client_id, is_bot):
    """This will be called whenever a player tries to connect. If the dispatcher
    returns False, it will not allow the player to connect and instead show them
    a message explaining why. The default message is "You are banned from this
    server.", but it can be set with :func:`minqlxtism.set_ban_message`.

    :param client_id: The client identifier.
    :type client_id: int
    :param is_bot: Whether or not the player is a bot.
    :type is_bot: bool

    """
    try:
        player = minqlxtism.Player(client_id)
        return minqlxtism.EVENT_DISPATCHERS["player_connect"].dispatch(player)
    except:
        minqlxtism.log_exception()
        return True

def handle_player_loaded(client_id):
    """This will be called whenever a player has connected and finished loading,
    meaning it'll go off a bit later than the usual "X connected" messages.
    This will not trigger on bots.

    :param client_id: The client identifier.
    :type client_id: int

    """
    try:
        player = minqlxtism.Player(client_id)
        return minqlxtism.EVENT_DISPATCHERS["player_loaded"].dispatch(player)
    except:
        minqlxtism.log_exception()
        return True

def handle_player_disconnect(client_id, reason):
    """This will be called whenever a player disconnects.

    :param client_id: The client identifier.
    :type client_id: int

    """
    try:
        player = minqlxtism.Player(client_id)
        return minqlxtism.EVENT_DISPATCHERS["player_disconnect"].dispatch(player, reason)
    except:
        minqlxtism.log_exception()
        return True

def handle_player_spawn(client_id):
    """Called when a player spawns. Note that a spectator going in free spectate mode
    makes the client spawn, so you'll want to check for that if you only want "actual"
    spawns.

    """
    try:
        player = minqlxtism.Player(client_id)
        return minqlxtism.EVENT_DISPATCHERS["player_spawn"].dispatch(player)
    except:
        minqlxtism.log_exception()
        return True

def handle_kamikaze_use(client_id):
    """This will be called whenever player uses kamikaze item.

    :param client_id: The client identifier.
    :type client_id: int

    """
    try:
        player = minqlxtism.Player(client_id)
        return minqlxtism.EVENT_DISPATCHERS["kamikaze_use"].dispatch(player)
    except:
        minqlxtism.log_exception()
        return True

def handle_kamikaze_explode(client_id, is_used_on_demand):
    """This will be called whenever kamikaze explodes.

    :param client_id: The client identifier.
    :type client_id: int
    :param is_used_on_demand: Non-zero if kamikaze is used on demand.
    :type is_used_on_demand: int


    """
    try:
        player = minqlxtism.Player(client_id)
        return minqlxtism.EVENT_DISPATCHERS["kamikaze_explode"].dispatch(player, True if is_used_on_demand else False)
    except:
        minqlxtism.log_exception()
        return True

def handle_console_print(text):
    """Called whenever the server prints something to the console and when rcon is used."""
    try:
        if not text:
            return

        # Log console output. Removes the need to have stdout logs in addition to minqlxtism.log.
        minqlxtism.get_logger().debug(text.rstrip("\n"))

        res = minqlxtism.EVENT_DISPATCHERS["console_print"].dispatch(text)
        if res is False:
            return False

        if _print_redirection:
            global _print_buffer
            _print_buffer += text

        if isinstance(res, str):
            return res

        return text
    except:
        minqlxtism.log_exception()
        return True

_print_redirection = None
_print_buffer = ""

def redirect_print(channel):
    """Redirects print output to a channel. Useful for commands that execute console commands
    and want to redirect the output to the channel instead of letting it go to the console.

    To use it, use the return value with the "with" statement.

    .. code-block:: python
        def cmd_echo(self, player, msg, channel):
            with minqlxtism.redirect_print(channel):
                minqlxtism.console_command("echo {}".format(" ".join(msg)))

    """
    class PrintRedirector:
        def __init__(self, channel):
            if not isinstance(channel, minqlxtism.AbstractChannel):
                raise ValueError("The redirection channel must be an instance of minqlxtism.AbstractChannel.")

            self.channel = channel

        def __enter__(self):
            global _print_redirection
            _print_redirection = self.channel

        def __exit__(self, exc_type, exc_val, exc_tb):
            global _print_redirection
            self.flush()
            _print_redirection = None

        def flush(self):
            global _print_buffer
            self.channel.reply(_print_buffer)
            _print_buffer = ""

    return PrintRedirector(channel)

def register_handlers():
    minqlxtism.register_handler("rcon", handle_rcon)
    minqlxtism.register_handler("client_command", handle_client_command)
    minqlxtism.register_handler("server_command", handle_server_command)
    minqlxtism.register_handler("frame", handle_frame)
    minqlxtism.register_handler("new_game", handle_new_game)
    minqlxtism.register_handler("set_configstring", handle_set_configstring)
    minqlxtism.register_handler("player_connect", handle_player_connect)
    minqlxtism.register_handler("player_loaded", handle_player_loaded)
    minqlxtism.register_handler("player_disconnect", handle_player_disconnect)
    minqlxtism.register_handler("player_spawn", handle_player_spawn)
    minqlxtism.register_handler("console_print", handle_console_print)

    minqlxtism.register_handler("kamikaze_use", handle_kamikaze_use)
    minqlxtism.register_handler("kamikaze_explode", handle_kamikaze_explode)
