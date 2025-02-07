use std::{fs, io, os::unix::net::UnixStream, path::Path};
use clap::Parser;

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to execute (default: code)
    #[arg(long, default_value = "code")]
    exec: String,

    /// Arguments to pass to the command
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    args: Vec<String>,
}

fn build_command_args(args: Args) -> (String, Vec<String>) {
    let mut command_args = vec![args.exec.clone()];
    command_args.extend(args.args);
    (args.exec, command_args)
}

fn is_like_vscode_ipc_socket(path: &Path) -> bool {
    if let Some(basename) = path.file_name() {
        if let Some(basename) = basename.to_str() {
            return basename.starts_with("vscode-ipc-") && basename.ends_with(".sock");
        }
    }
    false
}

fn test_socket_and_clean<P: AsRef<Path>>(path: P) -> Option<String> {
    let path = path.as_ref();
    if is_like_vscode_ipc_socket(path) {
        if UnixStream::connect(path).is_ok() {
            if let Some(str) = path.as_os_str().to_str() {
                return Some(str.to_owned());
            }
        }
        let _ = fs::remove_file(path);
    }

    None
}

fn find_best_match_and_clean(socket_path: &str) -> io::Result<Option<String>> {
    let path = Path::new(socket_path);
    if let Some(sock) = test_socket_and_clean(path) {
        return Ok(Some(sock));
    }

    if let Some(dir) = path.parent() {
        for p in dir.read_dir()?.flatten().map(|ent| ent.path()) {
            if let Some(sock) = test_socket_and_clean(&p) {
                return Ok(Some(sock));
            }
        }
    }

    Ok(None)
}

fn main() -> Result<(), exec::Error> {
    if let Ok(sock) = std::env::var("VSCODE_IPC_HOOK_CLI") {
        if let Ok(Some(sock)) = find_best_match_and_clean(&sock) {
            std::env::set_var("VSCODE_IPC_HOOK_CLI", sock);
        }
    } else {
        eprintln!("it seems you're not on remote.")
    }

    let args = Args::parse();
    let (exec, command_args) = build_command_args(args);
    Err(exec::execvp(&exec, command_args))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::net::UnixListener, path::Path};
    use uuid::Uuid;

    use crate::{find_best_match_and_clean, is_like_vscode_ipc_socket};

    fn gen_socket_name() -> String {
        format!("vscode-ipc-{}.sock", Uuid::new_v4())
    }

    #[test]
    fn test_is_like_vscode_ipc_socket() {
        let cases = [
            (
                "/tmp/vscode-ipc-7dc5f21f-5139-4eb4-8dd6-c4479f10c312.sock",
                true,
            ),
            (
                "/var/tmp/vscode-ipc-7dc5f21f-5139-4eb4-8dd6-c4479f10c312.sock",
                true,
            ),
            (
                "/vscode-ipc-7dc5f21f-5139-4eb4-8dd6-c4479f10c312.sock",
                true,
            ),
            (
                "/vscode-ipc-7dc5f21f-5139-4eb4-8dd6-c4479f10c312.txt",
                false,
            ),
            ("/vscode-7dc5f21f-5139-4eb4-8dd6-c4479f10c312.sock", false),
        ];

        for (idx, (path, expected)) in cases.into_iter().enumerate() {
            let path = Path::new(path);
            let got = is_like_vscode_ipc_socket(path);
            assert_eq!(
                got, expected,
                "#{}: expecting {:?} for {:?}, but got {:?}",
                idx, expected, path, got
            );
        }
    }

    #[test]
    fn find_best_match_and_clean_connects_to_vscode_ipc_hook_cli() {
        let tmpdir = tempfile::tempdir().unwrap();
        // let perm = fs::unix::metadata(tmpdir.path()).permissions().unwrap();
        let sock = tmpdir.path().join(gen_socket_name());
        let sock = sock.as_path().as_os_str().to_str().unwrap();
        let _listener = UnixListener::bind(sock).unwrap();

        let got = find_best_match_and_clean(sock);
        if let Ok(Some(n)) = &got {
            assert_eq!(n, sock)
        } else {
            panic!("expecting Ok(Some({:?})) but got {:?}", sock, got)
        }
    }

    #[test]
    fn find_best_match_and_clean_returns_err_when_fails_to_connect_to_vscode_ipc_hook_cli() {
        let tmpdir = tempfile::tempdir().unwrap();
        // let perm = fs::unix::metadata(tmpdir.path()).permissions().unwrap();
        let sock = tmpdir.path().join(gen_socket_name());
        let _ = UnixListener::bind(&sock).unwrap(); // shutdown

        let got = find_best_match_and_clean(sock.as_path().as_os_str().to_str().unwrap());
        if let Ok(None) = &got {
            // deletes the file
            assert!(!sock.exists())
        } else {
            panic!("expecting Ok(None) but got {:?}", got)
        }
    }

    #[test]
    fn find_best_match_and_clean_does_not_fail_through_by_scanning_the_parent_dir_of_vscode_ipc_hook_cli(
    ) {
        let tmpdir = tempfile::tempdir().unwrap();
        // let perm = fs::unix::metadata(tmpdir.path()).permissions().unwrap();
        let sock = tmpdir.path().join(gen_socket_name());
        let _ = UnixListener::bind(&sock).unwrap(); // shutdown
        let sock2 = tmpdir.path().join(gen_socket_name());
        let sock2 = sock2.as_os_str().to_str().unwrap();
        let _listener = UnixListener::bind(sock2).unwrap();

        let got = find_best_match_and_clean(sock.as_path().as_os_str().to_str().unwrap());
        if let Ok(Some(n)) = &got {
            // deletes the file
            assert!(!sock.exists());
            assert_eq!(n, sock2)
        } else {
            panic!("expecting Ok(Some({:?})) but got {:?}", sock2, got)
        }
    }

    #[test]
    fn test_build_command_args() {
        // 通常のケース
        let args = Args {
            exec: "cursor".to_string(),
            args: vec!["file.txt".to_string()],
        };
        let (exec, command_args) = build_command_args(args);
        assert_eq!(exec, "cursor", "exec command should be 'cursor'");
        assert_eq!(
            command_args,
            vec!["cursor", "file.txt"],
            "command args should contain the command name and file path"
        );

        // 引数なしのケース
        let args = Args {
            exec: "code".to_string(),
            args: vec![],
        };
        let (exec, command_args) = build_command_args(args);
        assert_eq!(exec, "code", "exec command should be 'code'");
        assert_eq!(
            command_args,
            vec!["code"],
            "command args should contain only the command name"
        );

        // 複数の引数があるケース
        let args = Args {
            exec: "cursor".to_string(),
            args: vec!["-r".to_string(), "file1.txt".to_string(), "file2.txt".to_string()],
        };
        let (exec, command_args) = build_command_args(args);
        assert_eq!(exec, "cursor", "exec command should be 'cursor'");
        assert_eq!(
            command_args,
            vec!["cursor", "-r", "file1.txt", "file2.txt"],
            "command args should contain all arguments in order"
        );
    }

    #[test]
    fn test_args_parsing() {
        // オプションを含むケース
        let args = Args::try_parse_from(["program", "--exec", "cursor", "-r", "--wait", "file.txt"]).unwrap();
        assert_eq!(args.exec, "cursor", "exec should be set to 'cursor'");
        assert_eq!(args.args, vec!["-r", "--wait", "file.txt"], "args should contain all options and the file path");

        // デフォルト値のテスト（オプション付き）
        let default_args = Args::try_parse_from(["program", "--wait", "file.txt"]).unwrap();
        assert_eq!(default_args.exec, "code", "exec should default to 'code'");
        assert_eq!(default_args.args, vec!["--wait", "file.txt"], "args should contain the option and file path");
    }
}
