use std::{
    collections::HashSet, fs::File, path::Path, 
    io::{Write, BufRead}, io,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}
};
use engage::{gamedata::{Gamedata, skill::SkillData, PersonData, item::ItemData}, mess::Mess};
use crate::randomizer::DVC_BLACK_LIST;

pub const NO_INHERITS: &[&str] = &[
    "SID_熟練者", "SID_熟練者＋", "SID_虚無の呪い", "SID_特効耐性", "SID_特効無効", "SID_不動",
    "SID_自壊", "SID_噛描", "SID_狂乱の一撃", "SID_バリア１",
    "SID_バリア２", "SID_バリア３", "SID_バリア４", "SID_バリア１_ノーマル用",
    "SID_バリア２_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア４_ノーマル用",
    "SID_チェインアタック威力軽減", "SID_チェインアタック威力軽減＋", "SID_自壊"
];

pub const PERSONAL_BLACK_LIST: &[&str] = &[
    "SID_瘴気の領域", "SID_異形狼連携", "SID_幻影狼連携", "SID_全弾発射",  "SID_守護者_E001",
    "SID_守護者_E002", "SID_守護者_E003", "SID_守護者_E004", "SID_守護者_使用不可"
];

pub const BLACKLIST_SKILL: &[&str] = &[
    "SID_バリア１", "SID_バリア２", "SID_バリア３", "SID_バリア４",
    "SID_バリア１_ノーマル用", "SID_バリア２_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア４_ノーマル用",
    "SID_異界の力_閉", "SID_異界の力_炎", "SID_異界の力_死", "SID_異界の力_夢", "SID_異界の力_科", "SID_守護者_E001",
    "SID_守護者_E002", "SID_守護者_E003", "SID_守護者_E004", "SID_計略", "SID_無し", "SID_切磋琢磨", "SID_オルタネイト", "SID_双聖", "SID_竜化_無効", "SID_虚無の呪い", "SID_異形狼連携", "SID_幻影狼連携", "SID_役に立ちたい_E001"
];

pub const BLACKLIST_ITEMS: &[&str] = &[
    "IID_マスタープルフ", "IID_リベラシオン改", "IID_リベラシオン改_ノーマル",
    "IID_リベラシオン", "IID_リベラシオン_M000", "IID_無し", "IID_不明", "IID_エンゲージ枠", "IID_火炎砲台", "IID_牙", "IID_邪竜石_E",
    "IID_邪竜石_E005", "IID_邪竜石_魔法攻撃_E", "IID_イル_反撃", "IID_イル_薙払いビーム", "IID_イル_突進",
    "IID_イル_吸収", "IID_イル_召喚", "IID_火のブレス", "IID_炎塊", "IID_ソンブル_物理攻撃",
    "IID_ソンブル_魔法攻撃", "IID_ソンブル_回転アタック", "IID_ソンブル_ビーム", "IID_ソンブル_エンゲージブレイク", "IID_ミセリコルデ",
];

pub struct DVCBlackLists {
    pub skill: DVCBlackList,
    pub no_inherits: DVCBlackList,
    pub personal_skill: DVCBlackList,
    pub job: DVCBlackList,
    pub recruitment_ignore: DVCBlackList,  // Ignore for Recruitment Order
    pub item: DVCBlackList,
}

pub struct DVCBlackList { pub indexes: HashSet<i32>, }

impl DVCBlackList {
    pub fn new() -> Self { Self { indexes: HashSet::new() } }
    pub fn from_slice<T: Gamedata>(slice: &[&str]) -> Self {
        Self { indexes: slice.iter().map(|x| T::get_index(x)).filter(|&x| x > 0).collect() }
    }
    pub fn load<T: Gamedata>(path: &str, default: Option<&[&str]>, desc: &str) -> Self {
        let mut indexes =
            if let Some(defaults) = default {
                defaults.into_iter().map(|x| T::get_index(x)).filter(|&x| x > 0).collect()
            }
            else { HashSet::new() };

        if let Ok(lines) = read_lines(path) {
            for line in lines.flatten() {
                if line.chars().nth(0).is_some_and(|c| c == '*') { continue; }
                line.split_whitespace().for_each(|x| {
                    let index = T::get_index(x);
                    indexes.insert(index);
                });
            }
        }
        else if let Some((mut file, defs)) = File::options().create(true).write(true).truncate(true).open(path).ok().zip(default) {
            writeln!(&mut file, "{}", desc).unwrap();
            defs.iter().for_each(|&item| { writeln!(&mut file, "{}", item).unwrap(); });
        }
        Self { indexes }
    }
    pub fn load_skill(path: &str, default: Option<&[&str]>, desc: &str) -> Self {
        let mut indexes = if let Some(defaults) = default { defaults.into_iter().flat_map(|x| SkillData::get(x).map(|s| s.parent.hash )).collect() } else { HashSet::new() };
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
        else if let Some((mut file, defs)) = File::options().create(true).write(true).truncate(true).open(path).ok().zip(default) {
            writeln!(&mut file, "{}", desc).unwrap();
            defs.iter().for_each(|&item| { writeln!(&mut file, "{}", item).unwrap(); });
        }
        Self { indexes }
    }
    pub fn allowed_index(&self, index: i32) -> bool { !self.indexes.contains(&index) }
}

impl DVCBlackLists {
    pub fn get_read() -> RwLockReadGuard<'static, DVCBlackLists>  { DVC_BLACK_LIST.get_or_init(|| RwLock::new(Self::init())).read().unwrap() }
    pub fn get_write() -> RwLockWriteGuard<'static, DVCBlackLists> { DVC_BLACK_LIST.get().unwrap().write().unwrap() }
    pub fn init() -> Self {
        let _ = std::fs::create_dir_all("sd:/engage/config/DVC");
        Self {
            personal_skill: DVCBlackList::load::<PersonData>("sd:/engage/config/DVC/person_skill.txt", Some(PERSONAL_BLACK_LIST), "** List of skills SIDs that unit's personal skill cannot randomized to **"),
            no_inherits: DVCBlackList::load_skill("sd:/engage/config/DVC/no_inherits.txt", Some(NO_INHERITS), "** List of skills SIDs that are excluded from chaos inheritance **", ),
            job: DVCBlackList::load_skill("sd:/engage/config/DVC/jobs.txt", None, "** List of class JIDs to exclude from unit class randomization **"),
            recruitment_ignore: DVCBlackList::load::<PersonData>("sd:/engage/config/DVC/recruitment_ignore.txt", None, "** List of PIDs that are not considered in randomizing recruitment order **", ),
            skill: DVCBlackList::load_skill("sd:/engage/config/DVC/skills.txt", Some(BLACKLIST_SKILL), "** List of Skill SIDs that are removed from the skill randomization pool **"),
            item: DVCBlackList::load::<ItemData>("sd:/engage/config/DVC/item.txt", Some(BLACKLIST_ITEMS), "** List of Item IIDs that are removed from the item randomization pool **")
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
