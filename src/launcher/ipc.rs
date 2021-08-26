use pop_launcher::{Request, Response};
use std::process;
use std::io::{self, BufRead, Write};

pub struct LauncherIpc {
    child: process::Child,
    stdin: process::ChildStdin,
    stdout: io::BufReader<process::ChildStdout>,
    exited: bool,
}

impl LauncherIpc {
    pub fn new() -> io::Result<Self> {
        let mut child = process::Command::new("pop-launcher")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or(
            io::Error::new(io::ErrorKind::Other, "failed to find child stdin")
        )?;

        let stdout = io::BufReader::new(child.stdout.take().ok_or(
                io::Error::new(io::ErrorKind::Other, "failed to find child stdout")
        )?);

        Ok(Self {
            child,
            stdin,
            stdout,
            exited: false,
        })
    }

    fn send_request(&mut self, request: Request) -> io::Result<()> {
        let mut request_json = serde_json::to_string(&request).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidInput, err)
        })?;
        request_json.push('\n');
        self.stdin.write_all(request_json.as_bytes())
    }

    fn recv_response(&mut self) -> io::Result<Response> {
        let mut response_json = String::new();
        self.stdout.read_line(&mut response_json)?;
        serde_json::from_str(&response_json).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, err)
        })
    }

    pub fn request(&mut self, request: Request) -> io::Result<Response> {
        self.send_request(request)?;
        self.recv_response()
    }

    //TODO: better exit implementation
    pub fn exit(&mut self) -> io::Result<Option<process::ExitStatus>> {
        if ! self.exited {
            self.send_request(Request::Exit)?;
            let status = self.child.wait()?;
            self.exited = true;
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }
}

impl Drop for LauncherIpc {
    fn drop(&mut self) {
        match self.exit() {
            Ok(_) => (),
            Err(err) => eprintln!("LauncherIpc::drop failed: {}", err),
        }
    }
}
