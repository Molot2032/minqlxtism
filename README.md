minqlxtism
======
minqlxtism is a modification of [minqlx](https://github.com/MinoMino/minqlx) that provides support for WASM plugins via [Extism](https://extism.org/docs/overview). Forked from the fantastic and [more actively maintained fork](https://github.com/MinoMino/minqlx/compare/master...tjone270:minqlxtended:master) of minqlx, [minqlxtism](https://github.com/tjone270/minqlxtended).

**🚧 Work in progress, nothing interesting here yet! 🚧**

Status
======
- [X] Hello WASM🌏!
- [ ] Plugin Manager
- [ ] Feature parity with minqlx's core API
- [ ] Reimplementation of base minqlx plugins. (MAYBE: py2wasm?)


Setup
============
1. Ensure the system is completely up-to-date:
  ```
  sudo apt update
  sudo apt upgrade -y
  ```

2. Install the git and build-essential packages:
  ```
  sudo apt-get -y install git build-essential
  ```

3. Install the Extism CLI and use it to install the Extism Runtime. The following command will install the CLI and runtime for your user only. 
  ```
  curl -s https://get.extism.org/cli | sh -s -- -v v1.6.2 -y -o $HOME/.local/bin && extism lib install --prefix ~/.local
  ```
  *v1.6.2 of the Extism CLI is selected to avoid [this issue](https://github.com/extism/cli/issues/115).*
  

4. Clone this repository and compile minqlxtism
  ```
  git clone https://github.com/Molot2032/minqlxtism.git
  cd minqlxtism
  make
  ```


Configuration
=============

- Copy everything from `minqlxtism/bin` into the Quake Live Dedicated Server's installation folder where `qzerodedx64` is located.
Use the provided `run_server_x_minqlxtism.sh` scripts to launch the server.

