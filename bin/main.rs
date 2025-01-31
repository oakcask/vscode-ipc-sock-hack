use std::{fs, io, os::unix::net::UnixStream, path::Path};

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

    for p in path.read_dir()?.flatten().map(|ent| ent.path()) {
        if let Some(sock) = test_socket_and_clean(&p) {
            return Ok(Some(sock));
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

    let argv = vec![String::from("code")];
    let argv = std::env::args().skip(1).fold(argv, |mut a, e| {
        a.push(e);
        a
    });

    Err(exec::execvp("code", argv))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::is_like_vscode_ipc_socket;

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
}
