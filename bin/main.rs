use std::{fs, io, os::unix::net::UnixStream, path::Path, process::exit};

fn is_like_vscode_ipc_socket(path: &Path) -> bool {
    if let Some(basename) = path.file_name() {
        if let Some(basename) = basename.to_str() {
            return basename.starts_with("vscode-ipc-") && basename.ends_with(".sock");
        }
    }
    return false;
}

fn test_socket_and_clean<P: AsRef<Path>>(path: P) -> Option<String> {
    let path = path.as_ref();
    if is_like_vscode_ipc_socket(path) {
        if let Ok(_) = UnixStream::connect(path) {
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

    for p in path
        .read_dir()?
        .into_iter()
        .filter(Result::is_ok)
        .map(|r| r.unwrap())
        .map(|ent| ent.path())
    {
        if let Some(sock) = test_socket_and_clean(&p) {
            return Ok(Some(sock));
        }
    }

    Ok(None)
}

fn main() -> ! {
    if let Ok(sock) = std::env::var("VSCODE_IPC_HOOK_CLI") {
        if let Ok(Some(sock)) = find_best_match_and_clean(&sock) {
            std::env::set_var("VSCODE_IPC_HOOK_CLI", sock);
            let mut argv = Vec::new();
            argv.push(String::from("code"));
            let argv = std::env::args().into_iter().fold(argv, |mut a, e| {
                a.push(e);
                a
            });
            let e = exec::execvp("code", argv);
            eprintln!("{:?}", e);
        }
    } else {
        eprintln!("it seems you're not on remote. exitting.")
    }
    exit(1)
}
