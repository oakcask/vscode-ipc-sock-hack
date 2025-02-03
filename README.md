# vscode-ipc-sock-hack

## What's this?

If you've ever seen a error like below, so you are the guest.

```
Error: connect ENOENT /tmp/vscode-ipc-7dc5f21f-5139-4eb4-8dd6-c4479f10c312.sock
    at PipeConnectWrap.afterConnect [as oncomplete] (node:net:1611:16) {
  errno: -2,
  code: 'ENOENT',
  syscall: 'connect',
  address: '/tmp/vscode-ipc-7dc5f21f-5139-4eb4-8dd6-c4479f10c312.sock'
}
error: There was a problem with the editor 'code -w'.
```

`vscode-ipc-sock-hack` is a shim command invokes `code` (vscode's CLI command),
but checks the socket files and cleans them up.

Be wary, this command keep trying to connect(2) to a file with pattern `vscode-ipc-*.sock`
where the `VSCODE_IPC_HOOK_CLI` points at, and files in the same directory too.
Those files will be deleted after failing to connect.

And please be noted: `vscode-ipc-sock-hack` doesn't aware whoever on the other side of socket.
So there may be a small gap to delete wrong file.
Yes, this is in rare case. Usually, those socket files are in `/tmp` with sticky bit
which prevents from deletion by users not the owner.

## Installation

```sh
# Option 1. use pre-built binary (currently supports aarch64-unknown-linux only.)
curl -sSL https://github.com/oakcask/vscode-ipc-sock-hack/releases/latest/download/aarch64-unknown-linux-gnu.tar.gz | tar zx -C /path/to/bin
```

```sh
# Option 2. build by yourself
cargo install --git https://github.com/oakcask/vscode-ipc-sock-hack.git
```

Aliasing is good hack to enable `vscode-ipc-sock-hack`,
which never invokes the login shell so this alias is just ignored
when `vscode-ipc-sock-hack` searches `code`.
Never causes infinite loop.

```sh
alias code='vscode-ipc-sock-hack'
```

If you are using `code -w` as EDITOR in vscode's intergrated terminal,
don't forget editing `settings.json`:

```diff
  {
    "terminal.integrated.env.linux": {
-       "EDITOR": "code -w"
+       "EDITOR": "vscode-ipc-sock-hack -w"
    }
  }
```
