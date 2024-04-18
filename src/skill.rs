use unity::prelude::*;
use unity::system::List;
use unity::il2cpp::object::Array;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    random::*,
    mess::*,
    gamedata::{unit::*, *, skill::*, GodData, god::*},
};
use std::fs::File;
use std::sync::Mutex;
use std::io::Write;
use super::CONFIG;
use crate::{person, deploy, utils::*, item::ENGAGE_ITEMS};

const EMBLEM_WEAPON: [i32; 19] = [2, 6, 66, 64, 2, 31, 18, 18, 10, 2, 514, 6, 28, 512, 14, 64, 64, 72, 66];
const STYLE: [&str;9] = ["None", "Cooperation", "Horse", "Covert", "Heavy", "Fly", "Magic","Prana", "Dragon"];
pub static SKILL_POOL: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
pub static MADDENING_POOL: Mutex<Vec<i32>> = Mutex::new(Vec::new());
static LEARN_SKILLS:  Mutex<Vec<i32>> = Mutex::new(Vec::new());
static LUNATIC_SKILLS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

static INHERIT_SKILLS:  Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());
static ENGAGE_SKILLS: Mutex<Vec<SkillIndex>> = Mutex::new(Vec::new());

static ENGAGE_ATTACKS: Mutex<Vec<EngageAttackIndex>> = Mutex::new(Vec::new());
static ENGAGE_ATK_SWAP: Mutex<Vec<EngageAttackIndex>> = Mutex::new(Vec::new());

//To Reset Syncho skills
static SYNCHO_SKILLS: Mutex<Vec<i32>> = Mutex::new(Vec::new());
static EIKIKA_ENGAGED: Mutex<Vec<i32>> = Mutex::new(Vec::new());

pub static STAT_BONUS:  Mutex<[i32; 66]> = Mutex::new([0; 66]);
static SYNCHO_RANDOM_LIST: Mutex<SynchoList> = Mutex::new(SynchoList { sync_list: Vec::new() });
pub static mut EIRIKA_INDEX: usize = 11;

pub const EMBLEM_ASSET: &[&str] = &["マルス", "シグルド", "セリカ", "ミカヤ", "ロイ", "リーフ", "ルキナ", "リン", "アイク", "ベレト", "カムイ", "エイリーク", "エーデルガルト", "チキ", "ヘクトル", "ヴェロニカ", "セネリオ", "カミラ", "クロム", "ディミトリ", "クロード"];

const BLACKLIST_SKILL: &[&str] = &[
    "SID_バリア１", "SID_バリア２", "SID_バリア３", "SID_バリア４",
    "SID_バリア１_ノーマル用", "SID_バリア２_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア４_ノーマル用",
    "SID_異界の力_閉", "SID_異界の力_炎", "SID_異界の力_死", "SID_異界の力_夢", "SID_異界の力_科", "SID_守護者_E001", 
    "SID_守護者_E002", "SID_守護者_E003", "SID_守護者_E004", "SID_計略", "SID_無し"];

const MADDENING_BLACK_LIST: &[&str] = &[
    "SID_杖使い＋＋", "SID_杖使い＋", "SID_杖使い", "SID_残像", "SID_日月の腕輪", "SID_慈悲", "SID_計略_引込の計", "SID_計略_猛火計", "SID_計略_聖盾の備え", "SID_計略_毒矢", 
    "SID_守護者", "SID_守護者_使用不可", "SID_全弾発射", "SID_輸送隊", "SID_裏邪竜ノ子_兵種スキル", "SID_負けず嫌い", "SID_竜脈・異", "SID_先生", "SID_増幅_闇", "SID_重唱", "SID_大好物",
    "SID_熟練者", "SID_ブレイク無効", "SID_師の導き", "SID_拾得", "SID_竜脈", "SID_特別な踊り", "SID_契約", "SID_七色の叫び＋", "SID_七色の叫び", "SID_戦場の花", "SID_平和の花", "SID_大盤振る舞い", "SID_料理再現",
    "SID_一攫千金", "SID_努力の才", "SID_白の忠誠", "SID_碧の信仰", "SID_緋い声援", "SID_筋肉増強剤", "SID_虚無の呪い", "SID_自壊", "SID_自己回復", "SID_角の睨み", "SID_囮指名", "SID_戦技", "SID_血統", 
    "SID_引き寄せ", "SID_体当たり", "SID_入れ替え", "SID_異形狼連携", "SID_幻影狼連携", "SID_星玉の加護"
];


const EIRIKA_TWIN_SKILLS: [&str; 12] = [ "SID_月の腕輪", "SID_太陽の腕輪", "SID_日月の腕輪", "SID_優風", "SID_勇空", "SID_蒼穹", "SID_月の腕輪＋", "SID_太陽の腕輪＋", "SID_日月の腕輪＋", "SID_優風＋", "SID_勇空＋", "SID_蒼穹＋" ];

pub struct SkillIndex {
    pub index: i32,
    pub in_use: bool,
    pub linked_use: bool,
}

impl SkillIndex {
    fn new(value: i32) -> Self { Self { index: value, in_use: false, linked_use: false, }}
}
struct EngageAttackIndex {
    pub index_1: i32,
    pub index_2: i32,
    pub in_use: bool,
    pub linked_use: bool,
}
impl EngageAttackIndex {
    fn new(value_1: i32, value_2: i32) -> Self { Self { index_1: value_1, index_2: value_2, in_use:false, linked_use: false,}}
}

pub struct SynchoSkill {
    pub index: i32,
    pub max_priority: i32,
    pub randomized_index: i32,
    pub in_use : bool,
    pub eirika_twin_skill: bool,
}
impl SynchoSkill {
    fn new(skill_index: i32, priority: i32, eirika: bool) -> Self {
        Self { index: skill_index, max_priority: priority, in_use: false, randomized_index: 0, eirika_twin_skill: eirika,} 
    }
}
pub struct SynchoList {
    pub sync_list: Vec<SynchoSkill>,
}

impl SynchoList {
    // For the three houses gambits to force them to be 4 separate skills instead of one 4-level skill
    pub fn add_to_non_upgrade(&mut self, sid: &str){
        let skill = SkillData::get(sid);
        if skill.is_none() { return; } 
        let skill_index = skill.unwrap().parent.index;
        let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
        if found.is_none() {  self.sync_list.push(SynchoSkill::new(skill_index, 0, false)); }
    }
    pub fn add_by_sid(&mut self, sid: &str){
        let skill = SkillData::get(sid);
        if skill.is_some() { self.add_list(skill.unwrap()); }
    }
    pub fn add_list(&mut self, skill: &SkillData) {
        if skill.get_flag() & 1 != 0 {  //must be not hidden
            return;
        }
        // ignore "None" "Night and Day", "Friendly Riviary"
        let sid = skill.sid.get_string().unwrap();
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨" { return; }
        if sid == "SID_無し" { return; }
        // if book of worlds
        if sid == "SID_異界の力" {
            let skill_index = skill.parent.index;
            let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
            if found.is_none() {  self.sync_list.push(SynchoSkill::new(skill_index, 0, false)); }
            return;
        }
        for x in 0..EIRIKA_TWIN_SKILLS.len() {  //Eirika Skills
            if EIRIKA_TWIN_SKILLS[x] == sid {
                if x < 6 {
                    let skill_index = skill.parent.index;
                    let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
                    if found.is_none() {  self.sync_list.push(SynchoSkill::new(skill_index, 1, true)); }
                }
                else {
                    let skill_index = skill.parent.index - 3;
                    let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
                    if found.is_some() { found.unwrap().max_priority = 2; }
                    else { 
                        self.sync_list.push(SynchoSkill::new(skill_index, 2, true)); }
                }
                return;
            }
        }

        let priority = skill.get_priority();
        if priority >= 1 {
            let skill_index = skill.parent.index - (priority - 1);
            let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
            if found.is_some() { found.unwrap().max_priority = priority; }
            else {
                self.sync_list.push(SynchoSkill::new(skill_index, priority, false)); }
        }
        else {
            let skill_index = skill.parent.index;
            let found = self.sync_list.iter_mut().find(|x| x.index == skill_index);
            if found.is_none() {
                self.sync_list.push(SynchoSkill::new(skill_index, priority, false)); 
            }
        }
    }
    pub fn reset(&mut self) {
        for x in self.sync_list.iter_mut() {
            x.in_use = false;
            x.randomized_index = 0;
        }
        self.sync_list[0].in_use = true;
    }
    pub fn randomized(&mut self, rng: &Random) {
        let size = self.sync_list.len() as i32;
        let s_list = &mut self.sync_list;
        // replace gambit
        let mut value =  rng.get_value( size - 5) + 1;
        while s_list[ value as usize].max_priority != 0 {
            value =  rng.get_value( size - 5) + 1;
        }
        s_list[0].randomized_index = value;
        s_list[ value as usize].in_use = true;

        for x in 1..size-4 {
            value = rng.get_value( size - 1  ) + 1 ;
            let max_priority = s_list[x as usize].max_priority;
            if max_priority == 0 {                 // non-upgradable -> non-ungradables
                while s_list[ value as usize ].in_use || s_list[ value as usize].max_priority != 0 {
                    value = rng.get_value( size - 1 ) + 1;
                }
                s_list[value as usize].in_use = true;
                s_list[x as usize].randomized_index = value;
            }
            else {
                while s_list[ value as usize ].in_use || s_list[ value as usize].max_priority == 0 { value = rng.get_value( size - 1) + 1;  }
                s_list[value as usize].in_use = true;
                s_list[x as usize].randomized_index = value;
            }
        }
    }
    pub fn get_replacement(&self, original_skill: &SkillData ) -> &'static SkillData {
        let skill_list = SkillData::get_list().unwrap();
        let o_skill = &skill_list[ original_skill.parent.index as usize];

        if original_skill.get_flag() & 1 != 0 {  //must be not hidden
            return o_skill;
        }
        // ignore "None" "Night and Day", "Friendly Riviary"
        let sid = original_skill.sid.get_string().unwrap();
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨" { return o_skill; }
        if sid == "SID_無し" { return o_skill; }

        let mut priority = original_skill.get_priority();
        let mut skill_index = original_skill.parent.index;
        let mut is_eirika_twin = false;
        if sid == "SID_異界の力" { priority = 0; }
        for x in 0..EIRIKA_TWIN_SKILLS.len() {  //Eirika Skills
            if EIRIKA_TWIN_SKILLS[x] == sid {
                if x < 6 { 
                    skill_index = original_skill.parent.index;
                    priority = 1;
                }
                else { 
                    skill_index = original_skill.parent.index - 3; 
                    priority = 2;
                }
                is_eirika_twin = true;
                break;
            }
        }
        if !is_eirika_twin {
            if priority == 0 { skill_index = original_skill.parent.index;}
            else { skill_index = original_skill.parent.index - (priority - 1); }
        }
        let found = self.sync_list.iter().find(|x| x.index == skill_index);
        if found.is_none() { return o_skill; }

        let new_skill_index = self.sync_list[ found.unwrap().randomized_index as usize ].index;
        let new_max_priority = self.sync_list[ found.unwrap().randomized_index as usize ].max_priority;
        is_eirika_twin = self.sync_list[ found.unwrap().randomized_index as usize ].eirika_twin_skill;

        if is_eirika_twin { /// Replacement skill is an Eirika Twin Skill (Lunar Brace/Solar Brace/Eclipse Brace etc...)
            if new_max_priority <= priority { // new_max_priority is 2 for Lunar/Solar/Eclipse Brace +
                let out_skill = &skill_list[ ( new_skill_index + 3 ) as usize]; 
                return out_skill;
            }
            else { // is Lunar/Solar/Eclipse
                let out_skill = &skill_list[ new_skill_index as usize];
                return out_skill;
            }
        }
        if new_max_priority == 0 || priority == 0 { 
            let out_skill = &skill_list[new_skill_index as usize]; 
            return out_skill;
         }
        if new_max_priority <= priority { 
            let out_skill = &skill_list[ (new_skill_index + new_max_priority - 1 ) as usize]; 
            return out_skill;
        }
        else { 
            let out_skill = &skill_list[ (new_skill_index + priority - 1  ) as usize];
            return out_skill;
        }
    } 
}


// For Engage Animation Swap
#[unity::class("App", "AssetTableConditionFlags")]
pub struct AssetTableConditionFlags {
    bits: &'static Array<u8>,
    pub keys: &'static mut List<Il2CppString>,
    pub hits: &'static List<i32>,
}

// Skill randomization and Emblem skills/stat boost randomization upon msbt loading
pub fn create_skill_pool() {
    // get skill index of hidden stat boost for emblems stat sync bonuses.
    for x in 0..11 {
        if x == 9 { continue; }
        for y in 1..7 {
            STAT_BONUS.lock().unwrap()[ 6*x + y-1 ] = find_emblem_stat_bonus_index(x as i32, y as i32);
        }
    }
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_計略");  //Add Gambit 
    let filename = "sd:/Draconic Vibe Crystal/Engage Weapon.txt";
    let mut f = File::options().create(true).write(true).truncate(true).open(filename).unwrap();
    for x in EMBLEM_ASSET {
        if *x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let keys = GodGrowthData::get_level_data(&growth_id);
        let god_name = Mess::get(GodData::get(&format!("GID_{}", x)).unwrap().mid).get_string().unwrap();
        if keys.is_some() {
            let level_data = keys.unwrap();
            let engage_skill = level_data[0].engage_skills.list.item[0].get_skill().unwrap();
            ENGAGE_SKILLS.lock().unwrap().push(SkillIndex::new(engage_skill.parent.index));
            for y in 0..level_data.len() {
                for z in 0..level_data[y].synchro_skills.list.size {
                    let skill = level_data[y].synchro_skills.list.item[z as usize].get_skill().unwrap();
                    let skill_index = skill.parent.index;
                    SYNCHO_SKILLS.lock().unwrap().push(skill_index);
                    SYNCHO_RANDOM_LIST.lock().unwrap().add_list(skill);
                }
                for z in 0..9 {
                    if level_data[y].style_names.items[z].len() != 0 {
                        let mut engage_weapon_string = format!("{} - Level {}, Count {}, Style {}:", god_name, y, level_data[y].style_names.count, STYLE[z as usize]);
                        for aa in 0..level_data[y].style_names.items[z].len() {
                            let item = level_data[y].style_names.items[z][aa].name;
                            engage_weapon_string = format!("{}\t{}", engage_weapon_string, Mess::get(item).get_string().unwrap());
                        }
                        writeln!(&mut f, "{}", engage_weapon_string).unwrap();
                    }
                }
            }
            if *x == "エイリーク" {
                for y in 0..level_data.len() {
                    for z in 0..level_data[y].engaged_skills.list.size {
                        let skill_index = level_data[y].engaged_skills.list.item[z as usize].get_skill().unwrap().parent.index;
                        EIKIKA_ENGAGED.lock().unwrap().push(skill_index);
                    }
                }
            }
        }
    }
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_勇空＋");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_太陽の腕輪＋");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_日月の腕輪＋");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_by_sid("SID_蒼穹＋");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_引込の計");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_猛火計");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_聖盾の備え");
    SYNCHO_RANDOM_LIST.lock().unwrap().add_to_non_upgrade("SID_計略_毒矢");
    println!("Size of Synchos {}", SYNCHO_SKILLS.lock().unwrap().len());
    println!("Generating skill pool");
    let skill_list = SkillData::get_list().unwrap();
    if SKILL_POOL.lock().unwrap().len() != 0 { return; }
    for x in 0..skill_list.len() {
        let sid = skill_list[x as usize].sid.get_string().unwrap();
        if sid == "SID_オルタネイト" || sid == "SID_切磋琢磨" { continue; }
        let mut skip = false;
        if BLACKLIST_SKILL.iter().find(|x| **x == sid ).is_some() { continue;}
         // Remove Night and Day and Friendly Riviary 
        let flag = skill_list[x].get_flag();
        if skill_list[x].help.is_none() { continue; }
        if skill_list[x].name.is_none() { continue; }
        if skill_list[x].get_inheritance_cost() != 0 { 
            let skill_name = Mess::get( skill_list[x].name.unwrap() ).get_string().unwrap();
            if skill_name.len() == 0 { continue;  }
            let skill_help = Mess::get( skill_list[x].help.unwrap() ).get_string().unwrap();
            if skill_help.len() == 0 { continue; }
            INHERIT_SKILLS.lock().unwrap().push(SkillIndex::new(x as i32));
        }
        if skill_list[x].is_style_skill() { continue; }
        for y in 0..8 {
            if flag & (1 << y ) != 0 {
                skip = true;
                break;
            }
        }
        if !skip {
            let skill_name = Mess::get( skill_list[x].name.unwrap() ).get_string().unwrap();
            if skill_name.len() == 0 { continue;  }
            let skill_help = Mess::get( skill_list[x].help.unwrap() ).get_string().unwrap();
            if skill_help.len() == 0 { continue; }
            if str_contains(skill_list[x].sid ,"E00"){ continue; }
            if str_contains(skill_list[x].sid ,"G00"){ continue; }
            if flag & 1 == 1 { continue; }
            SKILL_POOL.lock().unwrap().push(SkillIndex::new(x as i32));
            if MADDENING_BLACK_LIST.iter().find(|lol| **lol == sid ).is_none() {
                MADDENING_POOL.lock().unwrap().push(x as i32);
            }
        }
    }
    println!("Total Maddening Skills in Pool: {}",  MADDENING_POOL.lock().unwrap().len());
    println!("Total Skills in Pool: {}", SKILL_POOL.lock().unwrap().len());
    let job_list = JobData::get_list().unwrap();
    unsafe {
        for x in 0..job_list.len() {
            let job = &job_list[x];
            let mut index_learn = -1;
            let mut index_lunatic = -1;
            let learn_skill = job.learn_skill;
            if learn_skill.is_some() {
                let sid = learn_skill.unwrap();
                if !is_null_empty(sid, None) {  index_learn = SkillData::get_index(sid);  }
            }
            let lunatic_skill = job.lunatic_skill;
            if lunatic_skill.is_some() {
                let sid = lunatic_skill.unwrap();
                if !is_null_empty(sid, None) { index_lunatic = SkillData::get_index(sid);  }
            }
            LEARN_SKILLS.lock().unwrap().push( index_learn );
            LUNATIC_SKILLS.lock().unwrap().push( index_lunatic );
        }
    }
    let mut count = 0;
    for x in deploy::EMBLEM_GIDS {
        let god = GodData::get(*x).unwrap();
        let engage_index = SkillData::get_index( god.get_engage_attack() );
        if engage_index != -1 {
            ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(engage_index as i32, count));
        }
        unsafe {
            let link_engage = god_data_get_link_engage(god, None);
            if link_engage.is_some() {
                let link_index = SkillData::get_index( link_engage.unwrap() );
                if link_index != -1 {
                    ENGAGE_ATTACKS.lock().unwrap().push(EngageAttackIndex::new(link_index as i32, count));
                }
            }
        }
        count += 1;
    }
    println!("Number of Engage Attacks in pool: {}", ENGAGE_ATTACKS.lock().unwrap().len());
    println!("Number of Engage Skills in pool: {}", ENGAGE_SKILLS.lock().unwrap().len());
    for _x in 0..21 {
        ENGAGE_ATK_SWAP.lock().unwrap().push(EngageAttackIndex::new(0, 0));
    }
}

fn get_highest_priority(index: i32) -> i32 {
    let skill_list = SkillData::get_list().unwrap();
    let skill = &skill_list[ index as usize]; 
    let priority = skill.get_priority();
    if priority == 0 || priority > 9 { return index; }
    let sid = skill.sid.get_string().unwrap(); 
    //Eirika Skills
    for x in 0..EIRIKA_TWIN_SKILLS.len() {
        if sid == EIRIKA_TWIN_SKILLS[x as usize] {
            if x < 6 { return SkillData::get( EIRIKA_TWIN_SKILLS[ (6 + x) as usize]).unwrap().parent.index; }
            else { return SkillData::get( EIRIKA_TWIN_SKILLS[x as usize]).unwrap().parent.index; }
        }
    }
    let mut new_index = index+1;
    let mut current_priority = priority;
    loop {
        let new_skill = &skill_list[new_index as usize];
        if current_priority < new_skill.get_priority() {
            current_priority = new_skill.get_priority();
            new_index += 1;
        }
        else {
            println!("Highest Priority: {} -> {}", index, new_index-1);
            return new_index - 1; 
        }
    }
}


pub fn get_random_skill(difficulty: i32, rng: &Random) -> &'static SkillData {
    let skill_pool_size;
    let mut skill_index;
    let skill_list = SkillData::get_list().unwrap();
    if difficulty == 2 {
        skill_pool_size = MADDENING_POOL.lock().unwrap().len();
        skill_index = MADDENING_POOL.lock().unwrap()[ rng.get_value(skill_pool_size as i32) as usize];
        if GameVariableManager::get_bool("G_Cleared_M017") { skill_index = get_highest_priority(skill_index); }

        return &skill_list[ skill_index as usize];
    }
    else {
        skill_pool_size = SKILL_POOL.lock().unwrap().len();
        skill_index =  SKILL_POOL.lock().unwrap()[ rng.get_value(skill_pool_size as i32) as usize].index;
        return &skill_list[ skill_index as usize];
    }
}
pub fn reset_skills() {
    println!("Resetting skills to normal");
    let skill_pool_count = SKILL_POOL.lock().unwrap().len();
    let inherit_count = INHERIT_SKILLS.lock().unwrap().len();
    let engage_skill_count = ENGAGE_SKILLS.lock().unwrap().len();
    let engage_attack_count = ENGAGE_ATTACKS.lock().unwrap().len();
    let swap_count =  ENGAGE_ATK_SWAP.lock().unwrap().len();
    for x in 0..skill_pool_count { SKILL_POOL.lock().unwrap()[x as usize].in_use = false; }
    for x in 0..inherit_count { INHERIT_SKILLS.lock().unwrap()[x as usize].in_use = false; }
    for x in 0..engage_skill_count {  ENGAGE_SKILLS.lock().unwrap()[x as usize].in_use = false;   }
    for x in 0..engage_attack_count { 
        ENGAGE_ATTACKS.lock().unwrap()[x as usize].in_use = false;  
        ENGAGE_ATTACKS.lock().unwrap()[x as usize].linked_use = false;
    }
    for x in 0..swap_count {
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].index_1 = 0;
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].index_2 = 0;
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].in_use = false;  
        ENGAGE_ATK_SWAP.lock().unwrap()[x as usize].linked_use = false;  
    }
    // Reset Emblem Bond Data for Skills (Engaged/Engage/Sync)
    let mut index: usize = 0;
    let mut syncho_index: usize = 0;
    let skill_list = SkillData::get_list().unwrap();
    for x in EMBLEM_ASSET {
        if *x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let keys = GodGrowthData::get_level_data(&growth_id);
        if keys.is_some() {
            let level_data = keys.unwrap();
            let engage_skill = &skill_list[ ENGAGE_SKILLS.lock().unwrap()[index].index as usize ];
            if *x == "エイリーク" {     // Eirika Sync != Eirika Engaged 
                let mut eirika_index: usize = 0;
                for y in 0..level_data.len() {
                    level_data[y as usize ].engage_skills.replace(0, engage_skill, 5);
                    for z in 0..level_data[y].synchro_skills.list.size {
                        let synchro_skill = &skill_list[ SYNCHO_SKILLS.lock().unwrap()[syncho_index] as usize];
                        syncho_index += 1;
                        level_data[y as usize ].synchro_skills.replace(z as i32, synchro_skill, 5);
                    }
                    for z in 0..level_data[y].engaged_skills.list.size {
                        let engaged_skill = &skill_list[ EIKIKA_ENGAGED.lock().unwrap()[eirika_index] as usize];
                        level_data[y as usize ].engaged_skills.replace(z as i32, engaged_skill, 5);
                        eirika_index += 1;
                    }
                }
            }
            else {
                for y in 0..level_data.len() {
                    level_data[y as usize ].engage_skills.replace(0, engage_skill, 5);
                    for z in 0..level_data[y].synchro_skills.list.size {
                        let synchro_skill = &skill_list[ SYNCHO_SKILLS.lock().unwrap()[syncho_index] as usize];
                        syncho_index += 1;
                        level_data[y as usize ].synchro_skills.replace(z as i32, synchro_skill, 5);
                        level_data[y as usize].engaged_skills.replace(z as i32, synchro_skill, 5);
                    }
                }
            }
            ENGAGE_ATK_SWAP.lock().unwrap()[index].in_use = false;
        }
        index += 1;
    }
    SYNCHO_RANDOM_LIST.lock().unwrap().reset();
    // Engage Attack Reset Weapon Restrictions
    change_weapon_restrict("SID_マルスエンゲージ技", 2);    //Marth
    change_weapon_restrict("SID_シグルドエンゲージ技", 6);  //Sigurd
    change_weapon_restrict("SID_ロイエンゲージ技", 2);  // Roy
    change_weapon_restrict("SID_ルキナエンゲージ技", 2);    //Lucina
    change_weapon_restrict("SID_リンエンゲージ技", 16);     //Lyn
    change_weapon_restrict("SID_アイクエンゲージ技", 10);   //Ike
    change_weapon_restrict("SID_エイリークエンゲージ技", 2);    //Eirika
    change_weapon_restrict("SID_ヘクトルエンゲージ技", 10);     //Hector
    change_weapon_restrict("SID_カミラエンゲージ技", 8);    //Camilla   
    change_weapon_restrict("SID_クロムエンゲージ技", 2);    //Chrom
    unsafe { EIRIKA_INDEX = 11; }
}
pub fn replace_all_sid_person(person: &PersonData, sid: &Il2CppString, new_sid: &Il2CppString) {
    let person_list = PersonData::get_list_mut().unwrap();
    let name = person.get_name().unwrap().get_string().unwrap();
    let sid_comp = sid.get_string().unwrap();
    for x in 2..person_list.len() {
        let person_x = &person_list[x as usize];
        if person_x.parent.index == person.parent.index { continue;}
        if person_x.get_name().is_none() { continue; }
        if person_x.get_name().unwrap().get_string().unwrap() != name {continue; }
        if person_x.get_common_sids().is_none() { continue; }
        let personal_sid = person_x.get_common_sids().unwrap();
        for y in 0..personal_sid.len() {
            if personal_sid[y as usize].get_string().unwrap() == sid_comp {
                personal_sid[y as usize] = new_sid;
                person_x.on_complete();
                break;
            }
        }
    }
}

pub fn randomize_skills() {
    if !GameVariableManager::get_bool("G_Random_Skills") { return; }
    println!("randomizing skills");
    let skill_list = SkillData::get_list().unwrap();
    let rng = Random::instantiate().unwrap();
    let seed = 2*GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    let skill_pool_count = SKILL_POOL.lock().unwrap().len() as i32;
    for x in person::PIDS {
        let person = PersonData::get(x).unwrap();
        let personal_sid = person.get_common_sids().unwrap();
        let mut skill_index = rng.get_value(skill_pool_count) as usize;
        let mut index = SKILL_POOL.lock().unwrap()[skill_index].index as usize; 
        while SKILL_POOL.lock().unwrap()[skill_index].in_use || skill_list[index].get_inheritance_cost() != 0  { 
            skill_index = rng.get_value(skill_pool_count) as usize;
            index = SKILL_POOL.lock().unwrap()[skill_index].index as usize;
        }
        for y in 0..personal_sid.len() {
            let error_message = format!("{} missing skill in common sid index {}", person.get_name().unwrap().get_string().unwrap(), y);
            let skill = SkillData::get( &personal_sid[y as usize].get_string().unwrap() ).expect(&error_message);
            if skill.get_flag() & 1 == 0 {
                replace_all_sid_person(person, personal_sid[y as usize], skill_list[ index  as usize].sid);
                personal_sid[y as usize] = skill_list[ index  as usize].sid;
                SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
                break;
            }
        }
        person.on_complete();
    }
    println!("Person Skills complete");
    let job_list = JobData::get_list_mut().unwrap();
    for x in 0..job_list.len() {
        let job = &job_list[x as usize];
        let index_learn = LEARN_SKILLS.lock().unwrap()[x as usize];
        if index_learn == -1 { continue; }
        let mut skill_index = rng.get_value(skill_pool_count) as usize;
        let mut index = SKILL_POOL.lock().unwrap()[ skill_index  ].index as usize;
        while SKILL_POOL.lock().unwrap()[skill_index].in_use || skill_list[ index ].can_override_skill() { 
            skill_index = rng.get_value(skill_pool_count) as usize;
            index = SKILL_POOL.lock().unwrap()[ skill_index  ].index as usize;
        }
        job.set_learning_skill( skill_list[ index ].sid ); 
        SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
    }
    for x in 0..job_list.len() {
        let job = &job_list[x as usize];
        let index_lunatic = LUNATIC_SKILLS.lock().unwrap()[x as usize];
        if index_lunatic == -1 { continue; }
        let mut skill_index = rng.get_value(skill_pool_count) as usize;
        while SKILL_POOL.lock().unwrap()[skill_index].in_use {
            skill_index = rng.get_value(skill_pool_count) as usize;
        }
        let index = SKILL_POOL.lock().unwrap()[ skill_index ].index as usize;
        job.set_lunatic_skill( skill_list[ index ].sid ); 
        SKILL_POOL.lock().unwrap()[ skill_index ].in_use = true;
    }
    let maddening_pool_size = MADDENING_POOL.lock().unwrap().len() as i32;
    let mut ring_skill_set: [bool; 500] = [false; 500];
    let ring_list = RingData::get_list_mut().unwrap();
    for x in 0..ring_list.len() {
        if ring_list[x].rank != 3 { continue; }
        let equip_skills = ring_list[x].get_equip_skills();
        let mut index = rng.get_value(maddening_pool_size) as usize;
        while ring_skill_set[ index ] { index = rng.get_value(maddening_pool_size) as usize; }
        let s_index = MADDENING_POOL.lock().unwrap()[index];
        let s_high = get_highest_priority(s_index);
        equip_skills.clear();
        equip_skills.add_skill(&skill_list[s_high as usize],6, 0);
        ring_skill_set[ index ] = true;
    }
}

pub fn randomized_god_data(){
    let mode = GameVariableManager::get_number("G_Random_God_Mode");
    if mode == 0 { return; }
    //Engraves + 
    println!("Randomizing God Data...");
    let rng = Random::instantiate().unwrap();
    let seed = 3*GameVariableManager::get_number("G_Random_Seed") as u32;
    rng.ctor(seed);
    let skill_list = SkillData::get_list().unwrap();
    let mut weight: Vec<i8> = Vec::new();
    if mode == 1 || mode == 3 {
        let list_size = INHERIT_SKILLS.lock().unwrap().len();
        for x in 0..list_size { INHERIT_SKILLS.lock().unwrap()[x].in_use = false; }
        let mut max_engrave_stat: [i8; 5] = [0; 5];
        let mut min_engrave_stat: [i8; 5] = [100; 5];
        // get engrave min and max and change inheritable skills
        // Get List of Engage Skills to Randomized
        for x in deploy::EMBLEM_GIDS {
            let god = GodData::get(*x).unwrap();
            if max_engrave_stat[0] < god.get_engrave_avoid() { max_engrave_stat[0] = god.get_engrave_avoid(); }
            if max_engrave_stat[1] < god.get_engrave_critical() { max_engrave_stat[1] = god.get_engrave_critical();}
            if max_engrave_stat[2] < god.get_engrave_hit() { max_engrave_stat[2] = god.get_engrave_hit(); }
            if max_engrave_stat[3] < god.get_engrave_power() { max_engrave_stat[3] = god.get_engrave_power(); }
            if max_engrave_stat[4] < god.get_engrave_secure() { max_engrave_stat[4] = god.get_engrave_secure(); } 

            if max_engrave_stat[0] > god.get_engrave_avoid() { min_engrave_stat[0] = god.get_engrave_avoid(); }
            if max_engrave_stat[1] > god.get_engrave_critical() { min_engrave_stat[1] = god.get_engrave_critical();}
            if max_engrave_stat[2] > god.get_engrave_hit() { min_engrave_stat[2] = god.get_engrave_hit(); }
            if max_engrave_stat[3] > god.get_engrave_power() { min_engrave_stat[3] = god.get_engrave_power(); }
            if max_engrave_stat[4] > god.get_engrave_secure() { min_engrave_stat[4] = god.get_engrave_secure(); }  
            weight.push(god.get_engrave_weight().into());
            let ggid = GodGrowthData::try_get_from_god_data(god);
            if ggid.is_none() { continue; }
            let god_grow = ggid.unwrap(); 
            for y in 0..god_grow.len() {
                let level = god_grow[y].get_inheritance_skills();
                if level.is_none() {continue; }
                let inherit_skills = level.unwrap();
                for z in 0..inherit_skills.len() {
                    let mut value = rng.get_value(list_size as i32) as usize;
                    while INHERIT_SKILLS.lock().unwrap()[value].in_use { value = rng.get_value(list_size as i32) as usize; }
                    inherit_skills[z] = skill_list[ INHERIT_SKILLS.lock().unwrap()[value].index as usize ].sid;
                    INHERIT_SKILLS.lock().unwrap()[value].in_use = true;
                }
                god_grow[y].on_complete(); 
            }
        }
        // randomization of engrave data
        for x in 0..5 { 
            if x == 3 || x == 5 { continue; }
            max_engrave_stat[x] = 1 + ( max_engrave_stat[x] / 5); 
            min_engrave_stat[x] = (min_engrave_stat[x] / 5 ) - 1; 
        }
        for x in deploy::EMBLEM_GIDS {
            let god = GodData::get(*x).unwrap();
            for i in 0..5 {
                let mut value = 0;
                if i == 3 {
                    value = rng.get_min_max( min_engrave_stat[i as usize] as i32, max_engrave_stat[i as usize ] as i32) as i8;
                }
                else {
                    value = rng.get_min_max( 0 , max_engrave_stat[i as usize ] as i32) as i8;
                    let rng_value = rng.get_value(100);
                    if rng_value < 25 { value *= -5; }
                    else if  rng_value < 50 { value = 5* ( value / 2 ); }
                    else if rng_value < 75 { value = -5* (value / 2); }
                    else { value = 5*value; }
                }
                god.set_engrave( i as i32, value);
            }
            god.set_engrave(5, weight[ rng.get_value(weight.len() as i32) as usize ] as i8 );
            
        }
    }
    if mode == 2 || mode == 3 {
        let engage_atk_size = ENGAGE_ATTACKS.lock().unwrap().len();
        let mut linked_gid: [bool; 19] = [false; 19];
        let mut count = 0;
        for x in deploy::EMBLEM_GIDS { 
            let god = GodData::get(*x).unwrap();

            // Engage Attack
            let mut value = rng.get_value(engage_atk_size as i32) as usize;
            while ENGAGE_ATTACKS.lock().unwrap()[value].in_use { 
                value = rng.get_value(engage_atk_size as i32) as usize; 
            }
            let engage_sid = skill_list[ ENGAGE_ATTACKS.lock().unwrap()[value].index_1 as usize ].sid;
            god.set_engage_attack( engage_sid );
            ENGAGE_ATTACKS.lock().unwrap()[value].in_use = true;

            // Linked Emblem
            let mut linked_god_index = rng.get_value(19) as usize;
            while linked_gid[linked_god_index] || *x == deploy::EMBLEM_GIDS[linked_god_index] {
                linked_god_index = rng.get_value(19) as usize;
            }
            linked_gid[linked_god_index] = true;
            let gid_linked = deploy::EMBLEM_GIDS[linked_god_index];
            unsafe { god_data_set_link_gid(god, gid_linked.into(), None);  }

            // Linked Engage Attack
            let mut linked_index = rng.get_value(engage_atk_size as i32) as usize;
            while ENGAGE_ATTACKS.lock().unwrap()[linked_index].linked_use || linked_index == value {
                linked_index = rng.get_value(engage_atk_size as i32) as usize;
            }

            ENGAGE_ATK_SWAP.lock().unwrap()[count as usize].index_1 = ENGAGE_ATTACKS.lock().unwrap()[value].index_2;
            ENGAGE_ATK_SWAP.lock().unwrap()[count as usize].index_2 = ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_2; 
            let linked_engage_sid = skill_list[ ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_1 as usize ].sid;
            ENGAGE_ATTACKS.lock().unwrap()[linked_index].linked_use = true;

            unsafe { god_data_set_engage_link(god, linked_engage_sid, None);  }

            // If Edelgard then change it for Dimitri and Claude
            if *x == "GID_エーデルガルト" {
                let dimitri = GodData::get("GID_ディミトリ").unwrap();
                dimitri.set_engage_attack( engage_sid );
                let claude = GodData::get("GID_クロード").unwrap();
                claude.set_engage_attack( engage_sid );
                unsafe {
                    god_data_set_link_gid(dimitri, gid_linked.into(), None);
                    god_data_set_link_gid(claude, gid_linked.into(), None);
                    god_data_set_engage_link(dimitri, linked_engage_sid, None);
                    god_data_set_engage_link(claude, linked_engage_sid, None);
                    dimitri.on_complete();
                    claude.on_complete();
                    ENGAGE_ATK_SWAP.lock().unwrap()[19].index_1 = ENGAGE_ATTACKS.lock().unwrap()[value].index_2;
                    ENGAGE_ATK_SWAP.lock().unwrap()[19].index_2 = ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_2; 
                    ENGAGE_ATK_SWAP.lock().unwrap()[20].index_1 = ENGAGE_ATTACKS.lock().unwrap()[value].index_2;
                    ENGAGE_ATK_SWAP.lock().unwrap()[20].index_2 = ENGAGE_ATTACKS.lock().unwrap()[linked_index].index_2; 
                }
            }
            count += 1;
            god.on_complete();
        }
        if GameVariableManager::get_bool("G_Random_Engage_Weps") {
            let seed2 = 2*GameVariableManager::get_number("G_Random_Seed") as u32;
            let rng2 = Random::instantiate().unwrap();
            rng2.ctor(seed2);
            ENGAGE_ITEMS.lock().unwrap().randomize_list(rng2);
            ENGAGE_ITEMS.lock().unwrap().commit();
        }
        adjust_engage_weapon_type();
        // Random Engage Skills
    }
    randomize_engage_skills(rng);
    randomize_emblem_stat_bonuses(rng);
    randomized_emblem_syncho_skills(rng);
}
fn randomize_engage_skills(rng: &Random){
    if GameVariableManager::get_number("G_Random_God_Sync") <= 1 { return; }
    let skill_list = SkillData::get_list().unwrap();
    let mut count: usize = 0;
    for x in EMBLEM_ASSET {
        if *x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let keys = GodGrowthData::get_level_data(&growth_id);
        if keys.is_some() {
            let level_data = keys.unwrap();
            let mut index = rng.get_value(19) as usize;
            while ENGAGE_SKILLS.lock().unwrap()[index].in_use   { index = rng.get_value(19) as usize; }
            let engage_skill = &skill_list[ ENGAGE_SKILLS.lock().unwrap()[index].index as usize ];
            if engage_skill.sid.get_string().unwrap() == "SID_双聖" {
                unsafe {
                    EIRIKA_INDEX = count;
                }
            }
            for y in 0..level_data.len() {
                level_data[y as usize ].engage_skills.replace(0, engage_skill, 5);
            }
            ENGAGE_SKILLS.lock().unwrap()[index].in_use = true;
        }
        count += 1;
    }
}
fn randomize_emblem_stat_bonuses(rng: &Random){
    if GameVariableManager::get_number("G_Random_God_Sync") == 0 || GameVariableManager::get_number("G_Random_God_Sync") == 2 { return; }
    // Skill Range of Invisible Stat+ Skills
    let min_index = STAT_BONUS.lock().unwrap()[0]; //Lowest HP Index
    let max_index = STAT_BONUS.lock().unwrap()[65]; //Highest Move Index
    let skill_list = SkillData::get_list().unwrap();
    for x in EMBLEM_ASSET {
        if *x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
        let stats = get_stats_for_emblem(rng);
        for y in 0..level_data.len() {
            let mut stat_index: usize = 0;
            for z in 0..level_data[y].synchro_skills.list.size {
                let skill_index = level_data[y].synchro_skills.list.item[z as usize].get_skill().unwrap().parent.index;
                if skill_index <= max_index && min_index <= skill_index && stat_index < 4 {
                    let stat_skill = &skill_list[ skill_index as usize ];
                    let sb_index: usize = ( stats[ stat_index ] as usize ) * 6  + ( stat_skill.get_priority()  - 1 ) as usize; 
                    if sb_index >= 66 {
                        println!("Level {}, {}, Stat_Skill Priority {}", y,z,  stat_skill.get_priority())
                    }
                    let new_skill = &skill_list [ STAT_BONUS.lock().unwrap()[ sb_index ] as usize ];
                    level_data[y as usize ].synchro_skills.replace(z as i32, new_skill, 5);
                    stat_index += 1;
                }
            }
            stat_index = 0;
            for z in 0..level_data[y].engaged_skills.list.size {
                let skill_index = level_data[y].engaged_skills.list.item[z as usize].get_skill().unwrap().parent.index;
                if skill_index <= max_index && min_index <= skill_index && stat_index < 4 {
                    let stat_skill = &skill_list[ skill_index as usize ];
                    let sb_index: usize = ( stats[ stat_index ] as usize ) * 6  + ( stat_skill.get_priority()  - 1 ) as usize; 
                    let new_skill = &skill_list [ STAT_BONUS.lock().unwrap()[ sb_index ] as usize ];
                    level_data[y as usize ].engaged_skills.replace(z as i32, new_skill, 5);
                    stat_index += 1;
                }
            }
        }
    }
}

fn randomized_emblem_syncho_skills(rng: &Random) {
    if GameVariableManager::get_number("G_Random_God_Sync") <= 1 { return; }
    SYNCHO_RANDOM_LIST.lock().unwrap().randomized(rng);
    for x in EMBLEM_ASSET {
        if *x == "ディミトリ" { break; }
        let growth_id = format!("GGID_{}", x);
        let level_data = GodGrowthData::get_level_data(&growth_id).unwrap();
        for y in 0..level_data.len() {
            for z in 0..level_data[y].synchro_skills.list.size {
                let skill = level_data[y].synchro_skills.list.item[z as usize].get_skill().unwrap();
                let replacement_skill = SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill);
                level_data[y as usize ].synchro_skills.replace(z as i32, replacement_skill, 5);
            }
            for z in 0..level_data[y].engaged_skills.list.size {
                let skill = level_data[y].engaged_skills.list.item[z as usize].get_skill().unwrap();
                let replacement_skill =  SYNCHO_RANDOM_LIST.lock().unwrap().get_replacement(skill);
                level_data[y as usize ].engaged_skills.replace(z as i32, replacement_skill, 5);
            }
        }
    }
}

fn adjust_engage_weapon_type() {
    unsafe {
        for x in 0..19 {
            let weapon_mask_1 = ENGAGE_ITEMS.lock().unwrap().engage_weapon[x as usize];
            let engage_attack_sid = GodData::get(deploy::EMBLEM_GIDS[x as usize]).unwrap().get_engage_attack().get_string().unwrap();
            let mut weapon_mask_2 = EMBLEM_WEAPON[ x as usize];
            for y in 0..19 {
                let linked_engage_attack_sid = god_data_get_engage_link(GodData::get(deploy::EMBLEM_GIDS[y as usize]).unwrap(), None).unwrap().get_string().unwrap();
                if engage_attack_sid == linked_engage_attack_sid {
                    weapon_mask_2 = ENGAGE_ITEMS.lock().unwrap().engage_weapon[y as usize];
                    break;
                }
            }
            let combine_weapon_mask = weapon_mask_1 | weapon_mask_2;
            change_weapon_restrict(&engage_attack_sid, combine_weapon_mask);
        }
    }
}
fn change_weapon_restrict(sid :&str, value: i32) {
    let w1 = SkillData::get_mut(sid).unwrap().get_weapon_prohibit();
    if w1.value <= 2 { return; }
    w1.value = 1023 - value;
    let style = ["_連携", "_通常", "_通常", "_重装", "_飛行", "_魔法", "_通常", "_竜族", "＋", "＋_連携", "＋_通常", "＋_通常", "＋_重装", "＋_飛行", "＋_魔法", "＋_通常", "＋_竜族"];
    for x in style {
        let style_sid = format!("{}{}", sid, x);
        if SkillData::get(&style_sid).is_some() {
            let name = Mess::get(SkillData::get_mut(&style_sid).unwrap().name.unwrap()).get_string().unwrap();
            let w2 = SkillData::get_mut(&style_sid).unwrap().get_weapon_prohibit();
            println!("Engage Attack {} Weapon: {}", name, crate::utils::get_weapon_mask_str(value));    
            w2.value = 1023 - value;
        }
    }
}


#[skyline::hook(offset=0x01bb0440)]
pub fn asset_table_setup_hook_2(this: &mut AssetTableConditionFlags, state: i32, god: &GodData, is_darkness: bool, method_info: OptionalMethod) {
    // Fixing Engage Attack Animations when random
    for x in 0..21 {
        let string = format!("GID_{}", EMBLEM_ASSET[x]);
        if god.gid.get_string().unwrap() == string {
            unsafe { god_data_set_asset_id(god, EMBLEM_ASSET[x as usize].into(), None); }
            break;
        }
    }
    if state == 2 || state == 4 {
        if GameVariableManager::get_number("G_Random_God_Mode") <= 1 {  //No random Engage Attacks
            call_original!(this, state, god, is_darkness ,method_info);
            return;
        }
        let mut emblem_index = 0;
        for x in 0..21 {
            let string = format!("GID_{}", EMBLEM_ASSET[x]);
            if god.gid.get_string().unwrap() == string {
                emblem_index = x;
                break;
            }
        }
        if emblem_index >= 21 {
            call_original!(this, state, god, is_darkness ,method_info);
            return;
        }
        let asset_index;
        if Random::get_game().get_value(100) < 50 { asset_index = ENGAGE_ATK_SWAP.lock().unwrap()[emblem_index].index_1 as usize; }
        else { asset_index = ENGAGE_ATK_SWAP.lock().unwrap()[emblem_index].index_2 as usize;  }
        unsafe { god_data_set_asset_id(god, EMBLEM_ASSET[asset_index as usize].into(), None); }
    }
    call_original!(this, state, god, is_darkness ,method_info);
}


pub struct RandomSkillMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSkillMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_b(CONFIG.lock().unwrap().random_skill);
        if CONFIG.lock().unwrap().random_skill != result {
            CONFIG.lock().unwrap().random_skill  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_skill {  this.help_text = "Personals and class skills are randomized.".into(); }
        else { this.help_text = "No changes to personal and class skills.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_skill { this.command_text = "Player + Enemy".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}

pub struct RandomGodMod;
impl ConfigBasicMenuItemSwitchMethods for RandomGodMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_god_mode, 0, 3, 1);
        if CONFIG.lock().unwrap().random_god_mode != result {
            CONFIG.lock().unwrap().random_god_mode  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_mode == 1 {  this.help_text = "Engraves and inheritiable skills are randomized".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 2 { this.help_text = "Emblem link and engage attacks are randomized.".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 3 { this.help_text = "Engrave, inheritiable skills, and Engage attacks are all randomized.".into(); }
        else { this.help_text = "No changes to emblem data.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_mode == 1 { this.command_text = "Engraves and Inherits".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 2 { this.command_text = "Link Engage Attacks".into(); }
        else if CONFIG.lock().unwrap().random_god_mode == 3 { this.command_text = "Engraves, Inherits, Engage Atk".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}
pub struct RandomSynchoMod;
impl ConfigBasicMenuItemSwitchMethods for RandomSynchoMod {
    fn init_content(_this: &mut ConfigBasicMenuItem){}
    extern "C" fn custom_call(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        let result = ConfigBasicMenuItem::change_key_value_i(CONFIG.lock().unwrap().random_god_sync_mode, 0, 3, 1);
        if CONFIG.lock().unwrap().random_god_sync_mode != result {
            CONFIG.lock().unwrap().random_god_sync_mode  = result;
            Self::set_command_text(this, None);
            Self::set_help_text(this, None);
            this.update_text();
            return BasicMenuResult::se_cursor();
        } else {return BasicMenuResult::new(); }
    }
    extern "C" fn set_help_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_sync_mode== 1 {  this.help_text = "Emblem stat bonuses are randomized".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 2 { this.help_text = "Emblem sync and engage skills are randomized.".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 3 { this.help_text = "Emblem stats, sync, and engage skills are randomized.".into(); }
        else { this.help_text = "No changes to sync/engage emblem data.".into(); }
    }
    extern "C" fn set_command_text(this: &mut ConfigBasicMenuItem, _method_info: OptionalMethod){
        if CONFIG.lock().unwrap().random_god_sync_mode == 1 { this.command_text = "Stat Bonuses Only".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 2 { this.command_text = "Sync/Engage Skills".into(); }
        else if CONFIG.lock().unwrap().random_god_sync_mode == 3 { this.command_text = "All Sync Data".into(); }
        else { this.command_text = "No Randomization".into(); }
    }
}
#[no_mangle]
extern "C" fn skill_rnd() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomSkillMod>("Randomize Skills") } 
extern "C" fn god_rnd() -> &'static mut ConfigBasicMenuItem { ConfigBasicMenuItem::new_switch::<RandomGodMod>("Randomize Emblem Data") } 
pub fn install_skill_rnd() { 
    cobapi::install_global_game_setting(skill_rnd); 
    cobapi::install_global_game_setting(god_rnd);
}

#[unity::from_offset("App", "GodData", "get_EngageAttackLink")]
fn god_data_get_link_engage(this: &GodData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "GodData", "set_EngageAttackLink")]
fn god_data_set_engage_link(this: &GodData, value: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "GodData", "set_LinkGid")]
fn god_data_set_link_gid(this: &GodData, value: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "GodData", "set_AssetID")]
fn god_data_set_asset_id(this: &GodData, value: &Il2CppString, method_info: OptionalMethod);

#[unity::from_offset("App", "Unit", "GetEngageAttack")]
fn unit_get_engage_attack(this: &Unit, method_info: OptionalMethod) -> Option<&'static SkillData>;

#[unity::from_offset("App", "GodData", "get_EngageAttackLink")]
fn god_data_get_engage_link(this: &GodData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "SkillData", "get_WeaponProhibit")]
fn skill_data_get_weapon_prohibit(this: &SkillData, method_info: OptionalMethod) -> &'static mut WeaponMask;


