use std::{
    io::{Cursor, Read, BufRead},
    collections::HashSet, fs::File,  path::Path,
    sync::{RwLock, RwLockReadGuard, OnceLock},
    io
};
use engage::app::{ISkillDataMethods, IStructBase, IStructData_1Methods, ItemData, SkillData};
use crate::utils::to_opt;
pub static DVC_BLACK_LIST: OnceLock<RwLock<DVCBlackLists>> = OnceLock::new();

pub struct DVCBlackLists {
    pub skill: DVCBlackList,
    // pub no_inherits: DVCBlackList,
    pub personal_skill: DVCBlackList,
    pub item: DVCBlackList,
}
#[derive(Clone)]
pub struct DVCBlackList { pub indexes: HashSet<i32>, }

impl DVCBlackList {
    /*
    pub fn new() -> Self { Self { indexes: HashSet::new() } }
    pub fn from_slice<T: IStructData_1Methods<T>>(slice: &[&str]) -> Self {
        Self { indexes: slice.iter().map(|x| T::get_index((*x).into())).filter(|&x| x > 0).collect() }
    }
     */
    pub fn load<T: IStructData_1Methods<T>>(path: &str) -> Self {
        let mut indexes = HashSet::new();
        if let Ok(lines) = read_lines(path) {
            for line in lines.flatten() {
                if line.chars().nth(0).is_some_and(|c| c == '*') { continue; }
                line.split_whitespace().for_each(|x| {
                    let index = T::get_index(x.into());
                    indexes.insert(index);
                });
            }
        }
        Self { indexes }
    }
    pub fn load_skill(path: &str) -> Self {
        let mut indexes = HashSet::new();
        if let Ok(lines) = read_lines(path) {
            for line in lines.flatten() {
                if line.chars().nth(0).is_some_and(|c| c == '*') { continue; }
                line.split_whitespace().for_each(|x| {
                    if let Some(skill) = to_opt(SkillData::get(x.into())) {
                        if !skill.get_name().is_null() { println!("Found: {} to BlackList", skill.get_name()); }
                        indexes.insert(skill.hash());
                    }
                });
            }
        }
        Self { indexes }
    }
    pub fn allowed_index(&self, index: i32) -> bool { !self.indexes.contains(&index) }
}

impl DVCBlackLists {
    pub fn get_read() -> RwLockReadGuard<'static, DVCBlackLists>  { DVC_BLACK_LIST.get_or_init(|| RwLock::new(Self::init())).read().unwrap() }
    pub fn init() -> Self {
        let _ = std::fs::create_dir_all("sd:/engage/config/DVC");
        let mut blacklists  = [
            DVCBlackList::load::<ItemData>("sd:/engage/config/DVC/item.txt"),
            DVCBlackList::load_skill("sd:/engage/config/DVC/skills.txt"),
            DVCBlackList::load_skill("sd:/engage/config/DVC/person_skill.txt"),
            DVCBlackList::load_skill("sd:/engage/config/DVC/no_inherits.txt"),
        ];
        let mut black_list = Cursor::new(include_bytes!("../data/blacklist.bin"));
        let mut len: [u8; 4] = [0; 4];
        let mut v: [u8; 4] = [0; 4];
        black_list.read_exact(&mut len).unwrap();
        len.iter().enumerate()
            .for_each(|(i, &x)| {
                for _ in 0..x {
                    black_list.read_exact(&mut v).unwrap();
                    let hash = i32::from_be_bytes(v);
                    if i != 0 {
                        if let Some(skill) = to_opt(SkillData::try_get_from_hash(hash)) {
                            println!("Found Default Skill {} for BlackList", skill.get_name());
                        }
                    }
                    blacklists[i].indexes.insert(hash);
                }
            });
        println!("Item Black List: {}", blacklists[0].indexes.len());
        println!("Skill Black List: {}", blacklists[1].indexes.len());
        println!("Personal Skill Black List: {}", blacklists[2].indexes.len());
        Self {
            item: blacklists[0].clone(),
            skill: blacklists[1].clone(),
            personal_skill: blacklists[2].clone(),
            // no_inherits: blacklists[3].clone(),
        }
    }
    pub fn ignore_skill(&self, s: SkillData) -> bool { self.skill.indexes.contains(&s.hash()) || invalid_skill(s) }
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn invalid_skill(skill: SkillData) -> bool {
    skill.get_sid().to_rust_string().contains("00") &&
        skill.get_flag().value & 1023 != 0 || 
        skill.get_help().is_null() || skill.get_name().is_null() || skill.is_style_skill()
}
