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

`vscode-ipc-sock-hack` is a shim command invokes `code` (VSCode's CLI command),
but check the socket files and clean them up.

Be wary, this command keep trying to connect(2) to a file with pattern `vscode-ipc-*.sock`
where the `VSCODE_IPC_HOOK_CLI` points at, and files in the same directory.
Then files will be deleted if failed to connect.

## Installation

```
cargo install --git https://github.com/oakcask/vscode-ipc-sock-hack.git
alias code='vscode-ipc-sock-hack'
```
