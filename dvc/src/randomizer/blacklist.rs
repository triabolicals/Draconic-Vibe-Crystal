use std::{
    collections::HashSet, fs::File, path::Path, 
    io::{Write, BufRead}, io,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}
};
use std::io::{Cursor, Read};
use engage::{gamedata::{Gamedata, skill::SkillData, item::ItemData}, mess::Mess};
use crate::randomizer::DVC_BLACK_LIST;
pub struct DVCBlackLists {
    pub skill: DVCBlackList,
    pub no_inherits: DVCBlackList,
    pub personal_skill: DVCBlackList,
    pub item: DVCBlackList,
}
#[derive(Clone)]
pub struct DVCBlackList { pub indexes: HashSet<i32>, }

impl DVCBlackList {
    pub fn new() -> Self { Self { indexes: HashSet::new() } }
    pub fn from_slice<T: Gamedata>(slice: &[&str]) -> Self {
        Self { indexes: slice.iter().map(|x| T::get_index(x)).filter(|&x| x > 0).collect() }
    }
    pub fn load<T: Gamedata>(path: &str) -> Self {
        let mut indexes = HashSet::new();
        if let Ok(lines) = read_lines(path) {
            for line in lines.flatten() {
                if line.chars().nth(0).is_some_and(|c| c == '*') { continue; }
                line.split_whitespace().for_each(|x| {
                    let index = T::get_index(x);
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
                    if let Some(skill) = SkillData::get(x) {
                        indexes.insert(skill.parent.hash);
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
    pub fn get_write() -> RwLockWriteGuard<'static, DVCBlackLists> { DVC_BLACK_LIST.get_or_init(|| RwLock::new(Self::init())).write().unwrap() }
    pub fn init() -> Self {
        let _ = std::fs::create_dir_all("sd:/engage/config/DVC");
        let mut blacklists  = [
            DVCBlackList::load::<ItemData>("sd:/engage/config/DVC/item.txt"),
            DVCBlackList::load_skill("sd:/engage/config/DVC/skills.txt"),
            DVCBlackList::load_skill("sd:/engage/config/DVC/person_skill.txt"),
            DVCBlackList::load_skill("sd:/engage/config/DVC/no_inherits.txt"),
        ];
        let mut black_list = Cursor::new(include_bytes!("../../data/blacklist.bin"));
        let mut len: [u8; 4] = [0; 4];
        let mut v: [u8; 4] = [0; 4];
        black_list.read_exact(&mut len).unwrap();
        len.iter().enumerate().for_each(|(i, &x)| {
            for _ in 0..x {
                black_list.read_exact(&mut v).unwrap();
                let hash = i32::from_be_bytes(v);
                blacklists[i].indexes.insert(hash);
            }
        });
        Self {
            item: blacklists[0].clone(),
            skill: blacklists[1].clone(),
            personal_skill: blacklists[2].clone(),
            no_inherits: blacklists[3].clone(),

        }
    }
    pub fn ignore_skill(&self, s: &SkillData) -> bool { self.skill.indexes.contains(&s.parent.hash) || valid_skill(s) }
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn valid_skill(skill: &SkillData) -> bool {
   skill.sid.str_contains("00") ||
       skill.flag & 1023 != 0 ||
       skill.help.is_none_or(|h| Mess::get(h).to_string().len() < 2 ) ||
       skill.name.is_none_or(|h|Mess::get(h).to_string().len() < 2 ) ||
       skill.is_style_skill()
}
