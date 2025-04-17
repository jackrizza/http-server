use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Cmd {
    pub ssl: bool,
    pub path: String,
    pub password: String,
    pub thread_task: Option<thread::JoinHandle<()>>,
    pub logs: Arc<Mutex<Vec<String>>>,
    should_thread_end: Arc<Mutex<bool>>,
}

impl Clone for Cmd {
    fn clone(&self) -> Self {
        Self {
            ssl: self.ssl,
            path: self.path.clone(),
            password: self.password.clone(),
            thread_task: None,
            logs: self.logs.clone(),
            should_thread_end: self.should_thread_end.clone(),
        }
    }
}

impl Default for Cmd {
    fn default() -> Self {
        Self::new()
    }
}

impl Cmd {
    pub fn new() -> Self {
        Self {
            ssl: false,
            path: "".into(),
            password: "".into(),
            thread_task: None,
            logs: Arc::new(Mutex::new(Vec::new())),
            should_thread_end: Arc::new(Mutex::new(true)),
        }
    }

    pub fn start_thread(&mut self) {
        if self.thread_task.is_some() {
            return;
        }
        self.should_thread_end = Arc::new(Mutex::new(false));
        let self_ = self.clone();
        let log = self.logs.clone();
        let should_thread_end = self.should_thread_end.clone();
        let thread_task = thread::spawn(move || {
            log.lock().unwrap().push("Starting thread".into());
            self_.run(log);
            loop {
                if should_thread_end.lock().unwrap().clone() {
                    break;
                }
            }
        });
        self.thread_task = Some(thread_task);
    }

    pub fn end_thread(&mut self) {
        if self.thread_task.is_none() {
            return;
        }
        self.should_thread_end = Arc::new(Mutex::new(true));
    }

    fn run(&self, logger: Arc<Mutex<Vec<String>>>) {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &format!("cd {}", self.path)])
                .arg("dir .")
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(&format!("cd {}", self.path))
                .output()
                .expect("failed to execute process")
        };

        let change_dir = output.stdout;
        let change_dir = String::from_utf8(change_dir).unwrap();

        logger.lock().unwrap().push(change_dir);

        if !self.ssl {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", "http-server"])
                    .output()
                    .expect("failed to execute process")
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg("./http-server")
                    .stdout(Stdio::piped())
                    // creating a pipe to capture the standard error (stderr) of the child process.
                    .stderr(Stdio::piped())
                    .output()
                    .expect("failed to execute process")
            };

            let non_https_run = output.stdout;
            let non_https_run = String::from_utf8(non_https_run).unwrap();

            logger.lock().unwrap().push(non_https_run);
        } else {
            let output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", &format!("http-server -ap {}", &self.password)])
                    .output()
                    .expect("failed to execute process")
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(&format!("./http-server -ap {}", &self.password))
                    .output()
                    .expect("failed to execute process")
            };

            let https_run = output.stdout;
            let https_run = String::from_utf8(https_run).unwrap();

            logger.lock().unwrap().push(https_run);
        }
        // run_shell::ok()
    }
}
