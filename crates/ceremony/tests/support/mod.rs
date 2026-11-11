//! A committee of four `--new-identity` directories and its procedure, run in-process.
#![allow(dead_code)]

use ceremony::{hex, Report, FINISH};
use std::path::{Path, PathBuf};

pub const SIGNERS: [usize; 3] = [0, 2, 3];

pub fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("ceremony")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub struct Committee {
    pub root: PathBuf,
    pub identities: Vec<PathBuf>,
    pub printed: Vec<node::NewIdentity>,
    pub procedure: PathBuf,
}

pub fn procedure_text(printed: &[node::NewIdentity], slots: &[u32]) -> String {
    let list: Vec<String> = slots.iter().map(u32::to_string).collect();
    let mut text = format!(
        "threshold = 3\nsigners = [0, 2, 3]\nslots = [{}]\n",
        list.join(", ")
    );
    for p in printed {
        text += &format!(
            "\n[[member]]\nconsortium = \"{}\"\nhpke = \"{}\"\n",
            hex(&p.consortium),
            hex(&p.hpke)
        );
    }
    text
}

impl Committee {
    pub fn new(name: &str, slots: &[u32]) -> Self {
        let root = scratch(name);
        let identities: Vec<PathBuf> = (0..4).map(|i| root.join(format!("id-{i}"))).collect();
        let printed: Vec<_> = identities
            .iter()
            .map(|d| node::new_identity(d).unwrap())
            .collect();
        let procedure = root.join("procedure.toml");
        std::fs::write(&procedure, procedure_text(&printed, slots)).unwrap();
        Committee {
            root,
            identities,
            printed,
            procedure,
        }
    }

    pub fn dir(&self, ceremony: &str, i: usize) -> PathBuf {
        self.root.join(ceremony).join(format!("member-{i}"))
    }

    pub fn init(&self, ceremony: &str) -> Vec<Report> {
        std::fs::create_dir_all(self.root.join(ceremony)).unwrap();
        (0..4)
            .map(|i| {
                ceremony::init(&self.procedure, &self.identities[i], &self.dir(ceremony, i))
                    .unwrap()
            })
            .collect()
    }

    /// The files of `step` addressed to member `i`, from every member that wrote one.
    pub fn inbox(&self, ceremony: &str, step: u8, i: usize) -> Vec<PathBuf> {
        (0..4)
            .map(|j| {
                self.dir(ceremony, j)
                    .join("out")
                    .join(format!("{step:02}-{j}-{i}.msg"))
            })
            .filter(|p| p.exists())
            .collect()
    }

    pub fn step_one(&self, ceremony: &str, step: u8, i: usize) -> Result<Report, ceremony::Error> {
        let files = self.inbox(ceremony, step - 1, i);
        ceremony::step(&self.dir(ceremony, i), &self.identities[i], &files)
    }

    pub fn step_all(&self, ceremony: &str, step: u8) -> Vec<Report> {
        (0..4)
            .map(|i| self.step_one(ceremony, step, i).unwrap())
            .collect()
    }

    /// Steps 1 to `last` for every member; the reports of `last`.
    pub fn run(&self, ceremony: &str, last: u8) -> Vec<Report> {
        let mut reports = self.init(ceremony);
        for step in 2..=last.min(FINISH) {
            reports = self.step_all(ceremony, step);
        }
        reports
    }
}

pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
