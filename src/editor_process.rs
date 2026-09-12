use crate::{
    ipc,
    platform::{Command, Event},
};
use std::{
    io::{self, BufReader, BufWriter},
    path::Path,
    process::{Child, Stdio},
    sync::mpsc,
};

pub struct EditorProcess {
    pub child: Child,
    pub closing: bool,
    pub disconnected: bool,
    events: mpsc::Sender<Event>,
}

impl EditorProcess {
    pub fn spawn(executable: &Path, commands: mpsc::Sender<(u32, Command)>) -> io::Result<Self> {
        let mut child = std::process::Command::new(executable)
            .arg("--settings")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        let pid = child.id();
        let (events, receiver) = mpsc::channel();
        // Blocking pipe I/O never runs on the coordinator or input-hook thread.
        std::thread::spawn(move || {
            let mut writer = BufWriter::new(input);
            while let Ok(event) = receiver.recv() {
                if ipc::send(&mut writer, &event).is_err() {
                    break;
                }
            }
        });
        std::thread::spawn(move || {
            let mut reader = BufReader::new(output);
            while let Ok(Some(command)) = ipc::receive::<Command>(&mut reader) {
                if commands.send((pid, command)).is_err() {
                    return;
                }
            }
            let _ = commands.send((pid, Command::Disconnect));
        });
        Ok(Self {
            child,
            closing: false,
            disconnected: false,
            events,
        })
    }

    pub fn send(&self, event: Event) {
        let _ = self.events.send(event);
    }
}

impl Drop for EditorProcess {
    fn drop(&mut self) {
        // Only our settings child is owned here; never leave it orphaned on backend exit.
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
