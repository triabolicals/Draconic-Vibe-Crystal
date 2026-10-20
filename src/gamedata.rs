pub use engage::{
    ext::{List_1Ext, GameVariableManager},
    system::collections::generic::IList_1Methods,
    app::{
        structdictionary_1::*, structtemplate_1::*,
        structdataarraylist_1::*, structdata_1::*, structbase::*,
        goddata::*, unit::*, persondata::*, itemdata::*, random_2::*, skilldata::*, skillarray::*,
        jobdata::*, chapterdata::*, musicdata::*, interactdata::*, godgrowthdata::*, cookdata::*,
        ringdata::*, bitfield32::*, force::*, unitpool::*, godpool::*, godunit::*,
        ICapabilityBase_1Methods, Mess_IconCategory,
    },
    Mess,
};
use std::{
    collections::HashMap,
    sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard},
    io::{Cursor, Read}
};
use super::*;

mod skill;
mod emblem;
mod person;
mod items;
mod bondring;
mod job;
mod growth;
mod appearance;

pub use {
    person::PersonPool, job::*, appearance::AppearanceManager,
    growth::GrowthData, emblem::*, items::{ItemPool, *}, bondring::*, skill::*,
};

use outfit_core::Randomizer;
pub static RANDOMIZER_DATA: OnceLock<GameData> = OnceLock::new();
pub static RANDOMIZED_DATA: OnceLock<RwLock<RandomizedGameData>> = OnceLock::new();
use crate::{
    config::{DVCVariable, DVCFlags, DVCVariables},
    utils::to_opt, dvc, utils::{il2str}
};
use crate::dvc::sync::EmblemSkillPool;

pub struct GameData {
    pub bond_ring: Vec<BondRingData>,
    pub growth_data: GrowthData,
    pub interactions: Vec<i32>,
    pub skill_pool: SkillPool,
    pub item_pool: ItemPool,
    pub emblem_pool: EmblemPool,
    pub person_pool: PersonPool,
    pub job_db: JobDataBase,
    pub bgm: Vec<String>,
    pub chapter_rec_level: Vec<u8>,
}
impl GameData {
    pub fn init() -> Self {
        GodData::get_list().iter().filter(|g| g.get_gid().to_rust_string().contains("GID_相手"))
            .for_each(|g|{ 
                g.set_force_type(Force_Type::enemy());
                let gbid = g.get_gid().to_rust_string().replace("GID_", "GBID_");
                g.set_gbid(gbid);
            });
        
        let mut bgm =
        MusicData::get_list().iter().enumerate()
            .filter(|&(i, x)| i >= 6 && !il2str(x.get_event_name()).is_some_and(|c| c.contains("BGM_Sys")) && (!x.get_change_event_name().is_null() || (i >= 56 && i < 74) || i > 190))
            .flat_map(|(_, x)|{ il2str(x.get_change_event_name()).or_else(|| il2str(x.get_event_name())) })
            .collect::<Vec<String>>();
        bgm.push("BGM_Sys_Gmap".to_string());
        bgm.push("BGM_DLC_Sys_Gmap".to_string());
        bgm.push("BGM_Sys_Title_ST_Play".to_string());
        Self {
            bgm,
            emblem_pool: EmblemPool::init(),
            interactions: InteractData::get_list().iter().map(|data| data.get_flag().m_value()).collect(),
            chapter_rec_level: ChapterData::get_list().iter().map(|v| v.get_recommended_level()).collect(),
            bond_ring:  RingData::get_list().iter().map(|r|BondRingData::from_data(r)).collect(),
            person_pool: PersonPool::init(),
            growth_data: GrowthData::new(),
            item_pool: ItemPool::init(),
            skill_pool: SkillPool::init(),
            job_db: JobDataBase::init(),
        }
    }
    pub fn get() -> &'static Self { RANDOMIZER_DATA.get_or_init(||Self::init()) }
    pub fn get_item_pool() -> &'static ItemPool { &Self::get().item_pool }
    // pub fn get_random_skill(diff: i32, rng: Random_2) -> SkillData { Self::get().skill_pool.get_random_skill(diff, rng) }
    // pub fn get_random_skill_job(diff: i32, rng: Random_2, unit: Unit) -> Option<SkillData> { Self::get().skill_pool.get_random_skill_job(diff, rng, unit) }
    pub fn get_random_skill_dispos(diff: i32, rng: Random_2) -> SkillData { Self::get().skill_pool.get_random_skill_dispos(diff, rng) }
    pub fn get_random_item(ty: RandomItemReplacementType, with_rare: bool) -> Il2CppString { RANDOMIZER_DATA.get().unwrap().item_pool.random_item(ty, with_rare) }
    pub fn get_weapon_db() -> &'static WeaponDatabase { &Self::get().item_pool.weapon_db }
    pub fn update_personals(&self){
        let personal = DVCFlags::PersonalSkills.get_value();
        self.person_pool.playable.iter().for_each(|x|{ x.update_personal_skill(personal); });
        self.person_pool.enemy_character.iter().for_each(|x|{
           if let Some(p) =  x.get_enemy_replacement() {
               let enemy_data = x.get_data();
               if !enemy_data.is_null() {
                   if let Some(personal) = find_first_non_hidden(p.get_common_skills()){
                       set_first_non_hidden_index(enemy_data.get_common_skills(), personal.index());
                       set_first_non_hidden_index(enemy_data.get_mask_skill(), personal.index());
                   }
               }
           }
        });
    }
    pub fn random_refine(item: ItemData, index: i32) -> ItemData {
        let rng = DVCVariables::create_hash_rng(item.hash(), 2);
        let rank = item.m_weapon_level().value;
        let mut pool =
        ItemData::get_list().iter().filter(|i|{
            let rank_i = i.m_weapon_level().value;
            let use_type: i64 = 1 << (i.get_use_type().value as i64);
            i.get_flag().m_value() & 147 == 0 && (use_type & 0xC006E2 != 0) &&
            rank_i >= rank && rank_i <= (rank + 1) && i.get_range_o() < 12 && item.hash() != i.hash() && i.get_range_i() > 0
        }).collect::<Vec<_>>();
        let mut iteration = index + 1;
        loop {
            let new_item = pool.get_remove(rng);
            if iteration == 0 { return new_item.unwrap_or(item) }
            else { iteration -= 1; }
        }
    }
    pub fn get_playable_emblem_hashes() -> Vec<i32> {
        Self::get().emblem_pool.emblem_list.iter().enumerate().filter(|x| x.0 < 20 || x.0 >= 24).map(|x| *x.1).collect()
    }
    pub fn reset_interaction(&self) {
        InteractData::get_list().iter().zip(self.interactions.iter())
            .for_each(|(interaction, data)|{ interaction.get_flag().set_m_value(*data); });
    }
    pub fn update_bond_ring(&self) { randomize_bond_ring_skills(); }
    pub fn update_engraves(&self) {
        let engrave_level = DVCVariable::EngraveLevel.get_value();
        if engrave_level == 0 { self.emblem_pool.emblem_data.iter().for_each(|x|{ x.reset_engrave(); }) }
        else {
            let rng = DVCVariables::get_rng();
            self.emblem_pool.emblem_data.iter().map(|x| x.get_data())
                .for_each(|god|{
                    let (l, h) = match engrave_level {
                        2 => (-10, 50),
                        3 => (-30, 75),
                        4 => (-50, 100),
                        _ => (0, 25),
                    };
                    let power = engrave_power_dis(rng);
                    let crit = (rng.get_value_2(13) - 6)*5; // -30 to 30
                    let hit = (rng.get_value_2(10) - 3)*5;   // -15 to 30
                    let mut weight;
                    let mut dodge;
                    let mut avoid;
                    loop {
                        let rn = rng.get_value_2(20) - 6;
                        if rn < 0 { weight = 0; }
                        else {
                            if rn & 1 == 0 { weight = rn/2; }
                            else { weight = -rn/2; }
                        }
                        let total = 10*(power - weight);
                        if total >= l && total <= h { break; }
                    }
                    loop {
                        dodge = (rng.get_value_2(15) - 4)*5;    // -20 to 50
                        let total = dodge + crit;
                        if total >= l && total <= h { break; }
                    }
                    loop {
                        avoid = (rng.get_value_2(11) - 4)*5;    // -20 to 30
                        let total = hit + avoid;
                        if total >= l && total <= h { break; }
                    }
                    god.set_engrave_avoid(avoid as i8);
                    god.set_engrave_critical(crit as i8);
                    god.set_engrave_hit(hit as i8);
                    god.set_engrave_secure(dodge as i8);
                    god.set_engrave_power(power as i8);
                    god.set_engrave_weight(weight as i8);
                });
        }
    }
    pub fn get_interaction(&self) -> [i32; 20] {
        let mut out: [i32; 20] = [0; 20];
        let mode = DVCVariable::InteractSetting.get_value();
        match mode {
            2 => {  //Reverse
                for x in 0..10 {
                    out[x] = self.interactions[10 + x];
                    out[x + 10] = self.interactions[x];
                }
            },
            3 => {  //Self-Interaction
                for x in 0..10 {
                    out[x] = 1 << x;
                    out[x+10] = 1 << (x + 10);
                }
            },
            1 => {  // Random
                let rng = DVCVariables::get_rng();
                for x in 1..10 {
                    let mut kinds_1 = (1..10).into_iter().collect::<Vec<i32>>();
                    let mut kinds_2 = (1..10).into_iter().collect::<Vec<i32>>();
                    let mut chance = 100;
                    let mut value_a: i32 = 0;
                    let mut value_d: i32 = 0;
                    while rng.get_value_2(100) < chance {
                        if let Some(kind) = kinds_1.get_remove(rng) {
                            value_a |= 1 << kind;
                            value_d |= 1 << (kind + 10);
                            chance = chance >> 1;
                        }
                        else { break; }
                    }
                    chance = 100;
                    while rng.get_value_2(100) < chance {
                        if let Some(kind) = kinds_2.get_remove(rng) {
                            value_a |= 1 << (kind + 10);
                            value_d |= 1 << kind;
                            chance = chance >> 1;
                        }
                        else { break; }
                    }
                    out[x] = value_a;
                    out[x+10] = value_d;
                }
            },
            4 => {  // Fates Weapon Triangle
                out = [0, 36888, 24642, 67620, 329764, 286786, 299032, 0, 112, 0, 0, 24612, 67608, 36930, 37186, 67864, 24868, 0, 114688, 0];
            },
            5 => { for x in 0..20 { out[x] = 0; } }
            6 => { for x in 1..20 { out[x] = -1; } }
            _ => { for x in 0..20 { out[x] = self.interactions[x] } }
        }
        out
    }
    pub fn reset_engage_atk_mask(&self) {
        self.skill_pool.engage_attacks.iter().for_each(|(hash, weapon)|{
            if let Some(skill) = to_opt(SkillData::try_get_from_hash(*hash)){
                skill.m_style_skills().iter().for_each(|skill|{ skill.get_weapon_prohibit().set_m_value(*weapon); })
            }
        });
    }
    pub fn reset(&self) {
        self.item_pool.reset();
        self.reset_interaction();
        self.growth_data.reset();
        self.bond_ring.iter().for_each(|r|{ r.reset(); });
        self.emblem_pool.reset_all();
        self.job_db.reset();
        SkillPool::reset();
        PersonData::get_list().iter().for_each(|p|{ IPersonDataMethods::on_completed(p); });
        let god_lueur = GodData::try_get_from_hash(EmblemPool::EMBLEM_HASHES[19]);
        let lueur = PersonData::try_get_from_hash(PersonPool::LUEUR);
        lueur.set_link_god(god_lueur);
        ChapterData::get_list().iter()
            .zip(self.chapter_rec_level.iter())
            .for_each(|(chapter, level)|{chapter.set_recommended_level(*level); });
    }
}
fn engrave_power_dis(rng: Random_2) -> i32 {
    let mut rn= rng.get_value_2(45);
    let threshold = [13, 12, 8, 6, 4, 3, 2, 1];
    let mut out = 0;
    for x in threshold {
        if rn < x { break; }
        else {
            out += 1;
            rn -= x;
        }
    }
    if rng.get_value_2(2) == 0 { out = -out; }
    out
}
pub struct RandomizedGameData {
    pub engage_atks: EngageAttackRandomizer,
    pub engage_weapons: EngageItemRandomizer,
    pub engage_skills: EmblemSkillRandomizer,
    pub emblem_aptitude_randomizer: EmblemAptitudeRandomizer,
    pub refine: Vec<i32>,
    pub random_item_skills: HashMap<i32, RandomWeaponSkills>,
}

impl RandomizedGameData {
    pub fn get_read() -> RwLockReadGuard<'static, Self> { RANDOMIZED_DATA.get().unwrap().read().unwrap() }
    pub fn get_write() -> RwLockWriteGuard<'static, Self> { RANDOMIZED_DATA.get().unwrap().write().unwrap() }
    pub fn new(emblem: usize) -> Self {
        Self {
            refine: vec![-1; ItemData::get_count() as usize],
            engage_atks: EngageAttackRandomizer::new(emblem),
            engage_weapons: EngageItemRandomizer::init(),
            engage_skills: EmblemSkillRandomizer::init(),
            emblem_aptitude_randomizer: EmblemAptitudeRandomizer::new(emblem),
            random_item_skills: HashMap::new(),
        }
    }
    pub fn randomize(&mut self, data: &GameData) {
        let mut refine_set = data.item_pool.refine_iid.clone();
        refine_set.extend(data.item_pool.refine_iid.iter());
        refine_set.extend(data.item_pool.refine_iid.iter());
        self.refine.iter_mut().for_each(|v|{ *v = 0; });
        let rng = DVCVariables::get_rng();
        self.random_item_skills.clear();
        ItemData::get_list()
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_weapon() && x.get_flag().m_value() & 128 == 0 )
            .for_each(|(i, x)|{
                if let Some(v) = refine_set.get_remove(rng) { self.refine[i] = v; }
                let rng2 = DVCVariables::create_hash_rng(x.hash(), 2);
                if let Some(d) = RandomWeaponSkills::random_effect(x, rng2) { self.random_item_skills.insert(x.hash(), d); }
            });
        data.skill_pool.randomize(data);
        self.engage_atks.randomize(data);
        self.engage_skills.randomize(data);
        self.engage_weapons.randomize(data, &self.engage_atks);
        self.emblem_aptitude_randomizer.randomize();
    }
    pub fn update_weapon_stat_skills(&self, data: &GameData) {
        let rng = DVCVariables::get_rng();
        data.item_pool.stats.iter().for_each(|v|{ v.randomize(rng); });
        if DVCFlags::WeaponSkill.get_value() {
            self.random_item_skills.iter().for_each(|(k, v)|{
                let item = ItemData::try_get_from_hash(*k);
                v.add_random_skill(item);
            });
        }
    }
    pub fn update_engage_atk_items(&self, data: &GameData) {
        self.engage_atks.commit(data);
        self.engage_weapons.commit(data);
    }
    pub fn commit(&self, data: &GameData) {
        data.growth_data.update();
        data.job_db.update_attr();
        data.job_db.update_styles();
        data.get_interaction().into_iter().enumerate().for_each(|(i, f)|{ InteractData::try_get_2(i as i32).get_flag().set_m_value(f); });
        CookData::get_list().iter().for_each(|c|{ data.person_pool.randomize_cook_data(c); });
        self.engage_skills.commit(data);
        self.engage_skills.commit_stats(data);
        self.update_engage_atk_items(data);
        self.update_enemy_emblem(data);
        self.emblem_aptitude_randomizer.commit(data);
        data.update_personals();
        data.update_bond_ring();
        data.update_engraves();
        self.update_weapon_stat_skills(data);
        println!("Randomized Data committed");
    }
    pub fn update_enemy_emblem(&self, data: &GameData) {
        if !DVCFlags::RandomEnable.get_value() {
            data.emblem_pool.enemy_emblem.iter().for_each(|enemy|{
                enemy.reset();
                enemy.update_engage_atk();
            });
            return;
        }
        data.emblem_pool.enemy_emblem.iter().enumerate().for_each(|(enemy_index, enemy)|{
            let enemy_data = enemy.emblem_data.get_data();
            enemy.reset();
            enemy.update_engage_atk();
            let upgrade = enemy_index > 1;
            let randomized_index = enemy.get_replacement_index();
            if let Some(original_data) = enemy.get_replacement_source() {
                if let Some(original) = data.emblem_pool.emblem_data.iter().find(|v| v.hash == original_data.hash()){
                    if randomized_index != enemy.emblem_index {
                        enemy_data.set_female(original_data.get_female());
                        enemy_data.set_good_weapon(original_data.get_good_weapon());
                        enemy_data.set_face_icon_name_darkness(original_data.get_face_icon_name_darkness());
                        enemy_data.set_face_icon_name(original_data.get_face_icon_name());
                        let items = &original.level_data[0].style_item;
                        let sync_hash = [
                            get_sync_emblem_skill_hash(randomized_index as i32, 0, &data.emblem_pool),
                            get_sync_emblem_skill_hash(randomized_index as i32, 1, &data.emblem_pool),
                            get_sync_emblem_skill_hash(randomized_index as i32, 2, &data.emblem_pool),
                        ];
                        let n_skills = if enemy_index < 2 || sync_hash[1] == 0 { 1 } else if sync_hash[2] == 0 { 2 } else { 3 };
                        if let Some(level_data) = enemy.emblem_data.get_level_data() {
                            level_data.iter().enumerate().for_each(|(i, level)| {
                                let engage = level.get_engage_skills();
                                engage.clear();
                                let engage_skill = get_enemy_emblem_engage_skill(randomized_index as i32, i > 1, enemy_index > 1, &data.emblem_pool);
                                if engage_skill != 0 {
                                    if DVCVariable::EmblemEngageSkill.get_value() != 0 {
                                        engage.add_2(SkillData::try_get_2(self.engage_skills.get_engage_skill_index(engage_skill)), SkillData_Categorys::god(), 0);
                                    }
                                    else { engage.add_2(SkillData::try_get_2(engage_skill), SkillData_Categorys::god(), 0); }
                                }
                                let syncs = level.get_synchro_skills();
                                let engaged = level.get_engaged_skills();
                                syncs.clear();
                                engaged.clear();
                                let n = if i == 1 && n_skills == 3 { 2 } else { n_skills };
                                for x in 0..n {
                                    let mut hash = sync_hash[x];
                                    if hash == 0 { break; }
                                    if let Some(pos) = EmblemSkillPool::EIRIKA_SYNCS.iter().position(|v| *v == sync_hash[x]).filter(|v| *v < 6) {
                                        if upgrade { hash = EmblemSkillPool::EIRIKA_SYNCS[pos + 6]; }
                                    }
                                    else if upgrade {
                                        let skill = SkillData::try_get_from_hash(hash);
                                        if !skill.high_skill().is_null() { hash = skill.high_skill().hash(); }
                                    }
                                    let o_skill = SkillData::try_get_from_hash(hash);
                                    let index = o_skill.index();
                                    let skill = self.engage_skills.get_sync_replacement_skill(index);
                                    syncs.add_2(skill, SkillData_Categorys::god(), 0);
                                    if randomized_index == 11 {
                                        if x == 0 { 2 } else { 5 };
                                        let eirika_sync_index = if x == 0 { 2 } else { 5 } + if upgrade { 6 } else { 0 };
                                        let index = SkillData::try_get_from_hash(EmblemSkillPool::EIRIKA_SYNCS[eirika_sync_index]).index();
                                        engaged.add_2(self.engage_skills.get_sync_replacement_skill(index), SkillData_Categorys::god(), 0);
                                    }
                                    else { engaged.add_2(self.engage_skills.get_sync_replacement_skill(index), SkillData_Categorys::god(), 0); }
                                }
                                let style = level.get_style_engage_items();
                                style.m_items().iter().enumerate().for_each(|(i, style)| {
                                    let c = style.get_count() as usize;
                                    style.clear();
                                    for x in 0..c {
                                        let hash = items[3 * i + x];
                                        if hash != 0 { style.add(self.engage_weapons.get_replacement(hash)); }
                                    }
                                });
                            });
                        }
                    } else {
                        if let Some(level_data) = enemy.emblem_data.get_level_data() {
                            level_data.iter().enumerate().for_each(|(_, level)| {
                                let e = level.get_engaged_skills();
                                let s = level.get_synchro_skills();
                                let e2 = level.get_engage_skills();
                                let engaged: Vec<_> = e.m_list().iter().map(|v| self.engage_skills.get_sync_replacement_index(v.value as i32 & 0xFFF)).collect();
                                let sync: Vec<_> = s.m_list().iter().map(|v| self.engage_skills.get_sync_replacement_index(v.value as i32 & 0xFFF)).collect();
                                let engage: Vec<_> = e2.m_list().iter().map(|v| self.engage_skills.get_engage_skill_index(v.value as i32 & 0xFFF)).collect();
                                replace_skill_array(e, &engaged);
                                replace_skill_array(s, &sync);
                                replace_skill_array(e2, &engage);
                                let style = level.get_style_engage_items();
                                style.m_items().iter().for_each(|v| {
                                    let items: Vec<_> = v.iter().map(|v| self.engage_weapons.get_replacement(v.hash())).collect();
                                    v.clear();
                                    items.iter().for_each(|i|{v.add(*i); });
                                });
                            });
                        }
                    }
                }
            }
        });
        let m020 = GodData::get("GID_M020_敵セリカ".into());
        if m020.get_engage_attack().to_rust_string().contains("SID_セリカ") { m020.set_engage_attack("SID_セリカエンゲージ技_闇_M020"); }
        dvc::unit::ai::update_engage_attack_ai();
    }
}
pub fn get_enemy_emblem_engage_skill(index: i32, rampage: bool, dark: bool, emblem_pool: &EmblemPool) -> i32 {
    let skill_index = emblem_pool.emblem_data.get(index as usize)
        .and_then(|e| e.engage_skill_index.clone()).unwrap_or(0);
    match index {
        2|10|12 => { if rampage || !dark { skill_index } else { 0 } }
        1|3|4 => {
            if skill_index != 0 && dark {
                let skill = format!("{}_闇", SkillData::get_2(index).get_sid());
                let dark_skill = SkillData::get(skill.into());
                if !dark_skill.is_null() { dark_skill.index() } else { skill_index }
            }
            else { skill_index }
        }
        8|11 => 0,
        _ => { skill_index }
    }
}
pub const EMBLEM_SYNC_0: [i32; 57] = [
    -603137650, 768351239, 899393864, -1872726531, 1597801425, 1249281637,
    1706425885, 1428191993, 41966970, 188563602, 1350039474, 1238512915,
    -1158568192, -1535470911, 1107308702, 106021179, -679359997, 529226951, -505580457,

    29919765, -1650640255, 1467048018, 1723834087, 0, 2100465602,
    440730205, -896060322, 933055321, 0, 587764787, 1166279381,
    112775702, -610536379, 1304256837, -1255989066, 871049068, 432092216, 1920560263,

    -433480857, 191665449, 0, -1301523378, 0, 0,
    0, 0, 0, 0, 1349984976, 446418448,
    0, 1124471852, 311591062, 0, 0,0, 0
];
pub fn get_sync_emblem_skill_hash(index: i32, slot: i32, emblem_pool: &EmblemPool) -> i32 {
    if index < 19 {
        let i = slot * 19 + index;
        EMBLEM_SYNC_0[i as usize]
    }
    else { emblem_pool.emblem_data.get(index as usize).and_then(|v| v.syncs.get(slot as usize).cloned()).unwrap_or(0) }
}
pub fn replace_skill_array(array: SkillArray, indexes: &Vec<i32>) {
    indexes.iter().enumerate().for_each(|(index, idx)| {
        let mut s = array.m_list().get(index as i32);
        s.set_index(*idx);
        array.m_list().set_item(index as i32, s);
    })
}