use engage::{
    app::{assettable::*, BattleStyle, Gender},
    combat::characterappearance::*
};
use outfit_core::*;
use super::*;

use crate::dvc::asset::{new_accessory, print_asset_table_result};
use crate::enums::EMBLEM_GIDS;

pub static mut APPEARANCE: AppearanceManager = AppearanceManager::init();

pub struct AppearanceManager {
    pub playable_appearance: [(i16, i16); 100],
    pub npc_appearance: [(i16, i16); 30],
    pub emblem_appearance: [(i16, i16); 30],
    pub summon: [i32; 100],
    pub summon_person_index: [i32; 100],
    pub summon_count: i32,
}
#[allow(unused)]
impl AppearanceManager {
    pub const BLACK: [i32; 6] = [1187540167, 1187535337, -2009102214, -728744546, 1097589289, -1226236894];
    pub const RED_EYES: [i32; 7] = [ -1514923380, 1187540167, 1208025613, -392216545, -728744546, 1097589289, -1226236894];
    pub const HOODED: [i32; 2] = [-1722328881, -1402039598];
    pub const PAST_LUEUR: i32 = 1097589289;
    pub const fn init() -> Self {
        Self {
            playable_appearance: [(-1, -1); 100],
            npc_appearance: [(-1, -1); 30],
            emblem_appearance: [(-1, -1); 30],
            summon: [-1; 100],
            summon_person_index: [0; 100],
            summon_count: 0,
        }
    }
    pub fn get() -> &'static mut AppearanceManager { unsafe { &mut APPEARANCE } }
    pub fn randomize_summons(&mut self) {
        let rng = Random_2::get_system();
        let mut female: [bool; 100] = [false; 100];
        self.summon_count = 0;
        PersonData::get_list().iter().filter(|x| x.get_summon_rank().value > 0 && x.get_summon_rank().value < 3 )
            .enumerate()
            .for_each(|(i, person)|{
                if i < 100 {
                    female[i] = person.get_gender().value == 2;
                    self.summon_person_index[i] = person.index();
                    self.summon_count += 1;
                }
            });
        let mut male_set: Vec<usize> = PersonalDressData::get_male_indexes(false, |_| true);
        let mut female_set: Vec<usize> = PersonalDressData::get_female_indexes(false, |_| true);
        for i in 0..self.summon_count {
            if female[i as usize] { if let Some(idx) = female_set.get_remove(rng).map(|v| v as i32) { self.summon[i as usize] = idx; } }
            else { if let Some(idx) = male_set.get_remove(rng).map(|v| v as i32) { self.summon[i as usize] = idx; } }
        }
    }
    pub fn randomize(&mut self) {
        let mut total_m: Vec<usize> = PersonalDressData::get_male_indexes(true, |_| true);
        let mut total_f: Vec<usize> = PersonalDressData::get_female_indexes(true, |_| true);
        let mut emblem_m: Vec<usize> = PersonalDressData::get_male_indexes(true, |v| v.flags.contains(PersonalDressDataFlags::HasThumbnail) || v.flags.contains(PersonalDressDataFlags::HasBondFace));
        let mut emblem_f: Vec<usize> = PersonalDressData::get_female_indexes(true, |v| v.flags.contains(PersonalDressDataFlags::HasThumbnail) || v.flags.contains(PersonalDressDataFlags::HasBondFace));
        let rng = DVCVariables::get_rng();
        for x in 0..41 {
            self.playable_appearance[x] = (-1, -1);
            if let Some((male, female)) = total_m.get_remove(rng).zip(total_f.get_remove(rng)){
                self.playable_appearance[x] = (male as i16, female as i16);
            }
        }
        for x in 0..30 {
            self.emblem_appearance[x] = (-1, -1);
            if let Some((male, female)) = emblem_m.get_remove(rng).zip(emblem_f.get_remove(rng)) {
                self.emblem_appearance[x] = (male as i16, female as i16);
            }
        }
        total_m = PersonalDressData::get_male_indexes(false, |_| true);
        total_f = PersonalDressData::get_female_indexes(false, |_| true);
        for x in 0..30 {
            self.npc_appearance[x] = (-1, -1);
            if let Some((male, female)) = total_m.get_remove(rng).zip(total_f.get_remove(rng)){
                self.npc_appearance[x] = (male as i16, female as i16);
            }
        }
    }
    pub fn is_playable_rand_app(hash: i32) -> bool { UnitAssetMenuData::get_by_person_data(hash, false).is_some_and(|d| d.flag & 16 != 0) }
    pub fn get_assigned_appearance(app_index: i32, female: bool) -> Option<&'static PersonalDressData> {
        let appearance = Self::get();
        let i =
            if app_index < 100 { appearance.playable_appearance.get(app_index as usize) }
            else if app_index >= 141 && app_index < 170 { appearance.npc_appearance.get(app_index as usize-140) }
            else if app_index >= 200 && app_index < 300 { appearance.emblem_appearance.get(app_index as usize-200) }
            else { None }?;
        let i2 = if female { i.1 } else { i.0 };
        if i2 >= 0 { get_outfit_data().dress.personal.get(i2 as usize) }
        else { None }
    }
    pub fn get_default_appearance(person: PersonData) -> Option<&'static PersonalDressData> {
        let db = get_outfit_data();
        if let Some(appearance_index) = PersonPool::get_appearance_index(person) {
            match appearance_index {
                0 => { db.dress.get_personal_dress_by_person(person, DVCVariables::is_lueur_female()) }
                100..141 => {
                    let person = PersonPool::get_replacement(appearance_index  - 100, false, true).unwrap_or(person);
                    let female = if person.hash() == PersonPool::LUEUR { DVCVariables::is_lueur_female() } else { person.get_dress_gender().value == 2 };
                    db.dress.get_personal_dress_by_person(person, female)
                }
                _ => { db.dress.get_personal_dress_by_person(person, person.get_dress_gender().value == 2) }
            }
        }
        else { None }
    }
    pub fn get_appearance_person(person: PersonData, female: Option<bool>) -> Option<&'static PersonalDressData> {
        let hash = person.hash();
        let npc = DVCFlags::RandomBossesNPCs.get_value();
        if let Some(appearance_index) = PersonPool::get_person_index(person.hash()) {
            let female_1 =
                female.unwrap_or_else(||{
                    let g = if appearance_index == 0 { DVCVariable::LueurGender.get_value() } else { person.get_dress_gender().value };
                    g == 2
                });
            match appearance_index {
                0..100 => {
                    if Self::is_playable_rand_app(hash) { Self::get_assigned_appearance(appearance_index, female_1) } 
                    else { Self::get_default_appearance(person) }
                }
                100..141 => {
                    PersonPool::get_replacement(appearance_index - 100, false, false)
                        .or_else(|| PersonPool::get().playable.get(appearance_index as usize - 100).map(|p| p.get_person_data()))
                        .and_then(|person|{
                            let female = if person.hash() == PersonPool::LUEUR { DVCVariable::LueurGender.get_value() == 2 } else { person.get_dress_gender().value == 2 };
                            if Self::is_playable_rand_app(person.hash()) { Self::get_assigned_appearance(appearance_index,  female ) }
                            else { Self::get_default_appearance(person) }
                        })
                }
                141..145 => {
                    PersonPool::get_replacement(appearance_index - 104, false, true)
                        .and_then(|p| Self::get_appearance_person(p, None))
                        .or_else(||{
                            if npc { Self::get_assigned_appearance(appearance_index, female_1) }
                            else { Self::get_default_appearance(person) }
                        })
                }
                145..170 => {
                    if npc { Self::get_assigned_appearance(appearance_index, female_1) } else { Self::get_default_appearance(person) }
                }
                _ => { None }
            }
        }
        else {
            let d = Self::get();
            if npc {
                let index = person.index();
                if let Some(p) = d.summon_person_index.iter()
                    .position(|i| *i == index)
                    .and_then(|i| d.summon.get(i).filter(|&&i| i >= 0))
                {
                    get_outfit_data().dress.personal.get(*p as usize)
                }
                else { None }
            }
            else { None }
        }
    }
    pub fn get_appearance_name(person: PersonData, female: Option<bool>) -> Option<Il2CppString> {
        if PersonPool::is_playable(person) || PersonPool::is_nil(person) { None }   // Playable - No Name Switch
        else if let Some(p) = PersonPool::get_enemy_player_replacement(person, true){    // Playable Enemy Version
            if p.index() == 1 && !DVCVariables::is_main_menu() { Some(GameVariableManager::get_string(DVCVariables::LUEUR_NAME)) }
            else { Some(Mess::get(p.get_name())) }
        }
        else { Self::get_appearance_person(person, female).map(|a| a.get_name()) }  // Everyone else Switch
    }
    pub fn get_emblem_appearance_name(emblem_idx: i32) -> Il2CppString {
        if emblem_idx == 19 {
            if let Some(hero) = to_opt(UnitPool::get_hero(false)) { hero.get_name() }
            else if let Some(hero) = PersonPool::get_replacement(0, false, false) { Mess::get(hero.get_name()) }
            else { GameVariableManager::get_string(DVCVariables::LUEUR_NAME) }
        }
        else {
            let god = GodData::get(EMBLEM_GIDS[emblem_idx as usize].into());
            if DVCFlags::GodNames.get_value() {
                let idx = Self::get().emblem_appearance[emblem_idx as usize];
                let app = if god.get_female() == 1 { idx.1 } else { idx.0 };
                if app >= 0 { get_outfit_data().dress.personal[app as usize].get_name() }
                else { Mess::get(god.get_mid()) }
            }
            else { Mess::get(god.get_mid()) }
        }
    }
    pub fn get_appearance_emblem(hash: i32, dark: bool) -> Option<&'static PersonalDressData> {
        let emblem_index = EmblemPool::get_emblem_index(hash).filter(|i| *i < 24)?;
        Self::get_appearance_emblem_2(emblem_index, dark)
    }
    pub fn get_appearance_emblem_2(idx: i32, dark: bool) -> Option<&'static PersonalDressData> {
        let m = Self::get();
        let db = get_outfit_data();
        if idx == 19 {
            if let Some(hero) = PersonPool::get_replacement(0, false, false) {
                Self::get_appearance_person(hero, None)
            }
            else {
                let lueur = PersonData::try_get_2(1);
                Self::get_appearance_person(lueur, None)
            }
        }
        else if idx < 30 {
            let god = GodData::try_get_from_hash(EmblemPool::EMBLEM_HASHES[ idx as usize]);
            if god.is_null() { None }
            else {
                let hash = god.hash();
                let female = god.get_female() == 1;
                let mut app_idx = -1;
                if DVCFlags::GodNames.get_value() {
                    let i = m.emblem_appearance[idx as usize];
                    app_idx = if female { i.1 } else { i.0 };
                }
                if app_idx >= 0 { db.dress.personal.get(app_idx as usize) }
                else if dark {
                    let hash = if idx < 12 || idx == 23 { hash } else { GodData::get(format!("GID_E006_{}", EmblemPool::GIDS[idx  as usize]).into()).hash() };
                    db.dress.personal.iter().find(|v| v.flags.contains(PersonalDressDataFlags::Dark) && v.hash == hash)
                }
                else { db.dress.personal.iter().find(|v| v.flags.contains(PersonalDressDataFlags::FromGod) && v.hash == hash) }
            }
        }
        else { None }
    }
    pub fn get_person_appearance(unit: Unit, result: AssetTable_Result, mode: i32){
        let person = unit.get_person();
        let db = get_outfit_data();
        // let gender = unit.get_dress_gender().value == 2;
        let female = db.get_dress_gender(if mode == 2 { result.get_dress_model() } else { result.get_body_model() });
        if let Some(data) = Self::get_appearance_person(unit.get_person(), Some(female.value == 2)) {
            if PersonPool::is_playable(person){
                if Self::is_playable_rand_app(person.hash()) { data.apply_appearance(result, mode, false, None, &db.hashes, true); }
                else if unit.get_job().hash() == GameVariableManager::get_number(format!("G_JG_{}", person.get_pid())) {
                    if mode == 2 { if let Some(ubody) = db.try_get_asset(AssetType::Body, data.ubody) { result.set_dress_model(ubody.as_str()); } }
                    else { if let Some(obody) = db.hashes.get_obody(data.ubody) { result.set_body_model(obody); } }
                }
            }
            else { data.apply_appearance(result, mode, false, None, &db.hashes, true); }
        }
        else if DVCFlags::RandomBossesNPCs.get_value(){  // Summons
            let person_idx = person.index();
            let m = Self::get();
            if let Some(data) = m.summon_person_index.iter().position(|i| *i == person_idx)
                .map(|i| m.summon_person_index[i]).filter(|i| *i >= 0)
                .and_then(|idx| db.dress.personal.get(idx as usize))
            {
                data.apply_appearance(result, mode, false, None, &db.hashes, true);
            }
        }
    }
    pub fn set_rand_emblem_appearance(god_data: GodData, result: AssetTable_Result, mode: i32, dark: bool) -> bool {
        if !DVCFlags::GodNames.get_value() && !god_data.is_hero() {
            let hash = god_data.hash();
            if let Some(pos) = EmblemPool::EMBLEM_HASHES.iter().position(|h| *h == hash) {
                if ((pos < 12 || pos == 23) && dark) || !dark { return false; }
            }
            let gid = god_data.get_gid().to_rust_string();
            if gid.contains("GID_相手") && !gid.contains("リュール") { return false;}
        }
        let db = get_outfit_data();
        let is_enemy_dark = EmblemPool::is_enemy_dark(god_data);
        let dark = dark | is_enemy_dark;
        let mut replaced = false;
        if let Some(app) = EmblemPool::get_appearance_index(god_data.hash()).and_then(|v|Self::get_appearance_emblem_2(v, dark)) {
            if app.flags.contains(PersonalDressDataFlags::FromPerson) && app.flags.contains(PersonalDressDataFlags::HasJob){
                if let Some(person) = to_opt(PersonData::try_get_from_hash(app.hash)){
                    let item_kind = person.get_job().get_equipable_weapon_kinds().get(0);
                    let item = GameData::get_item_pool().weapon_db.get_random_weapon(None, item_kind.value, 1, true).unwrap_or(ItemData::null());
                    result.setup_5(AssetTable_Modes{value: mode}, person, person.get_job(), item, CharacterAppearance::conditions());
                    print_asset_table_result(result, 10 + mode);
                    replaced = true;
                }
            }
            if !replaced { app.apply_appearance(result, mode, false, None, &db.hashes, true); }
        }
        let mut conditions = AssetTable::s_condition_flags().m_bits();
        let demo = get_condition_index("デモ").unwrap_or(0);
        let info = get_condition_index("情報").unwrap_or(0);
        let talk = get_condition_index("会話").unwrap_or(0);
        let rampage = get_condition_index("暴走").unwrap_or(0);
        let demo = conditions.get(demo);
        let info = conditions.get(info);
        let talk = conditions.get(talk);
        let rampage = conditions.get(rampage);
        let mut emblem_aura = if mode == 1 { "oAcc_Eff_EmblemAura-0" } else { "uAcc_Eff_EmblemAura-0"}.to_string();

        if dark || (god_data.get_force_type().value != 0 && !god_data.get_gid().to_rust_string().contains("相手")){
            let mut sound = result.get_sound();
            sound.voice_id = "_Blank".into();
            sound.footstep_id = "Emblem".into();
            result.set_sound(sound);
            if !conditions.get(info) || !conditions.get(talk) || !conditions.get(demo) {
                result.commit_8(new_accessory("c_hip_jnt", "null"));
            }
            if rampage { emblem_aura += "9-00"; } else { emblem_aura += "2-00"; }
            unity::field_set_value_at_offset::<i32>(result, 0x14c, 2);
            result.set_hair_color(engage::unity_engine::Color {r: 0.63, g: 0.13, b: 0.13, a: 1.0});
            result.set_grad_color(engage::unity_engine::Color {r: 0.63, g: 0.13, b: 0.13, a: 1.0});
            result.commit_8(new_accessory("c_trans", emblem_aura));
        }
        else {
            if !demo && !talk && !info { emblem_aura += "1-00" }
            else if demo { emblem_aura += "1-01" }
            else if talk || info { emblem_aura += "1-02" }
            unity::field_set_value_at_offset::<i32>(result, 0x14c, 3);
            result.commit_8(new_accessory("c_hip_jnt", emblem_aura.as_str()));
        }
        result.replace(AssetTable_Modes{value: mode});
        replaced
    }
    pub fn tiki_dragon(result: AssetTable_Result, unit: Unit, is_dark: bool, mode: i32) {
        let search_list = AssetTable::s_search_lists();
        let con = get_condition_index(if is_dark { "PID_E001_Boss_竜化" } else {"EID_チキ"}).unwrap();
        let voice = outfit_core::il2str(result.get_sound().voice_id);
        if let Some(entry) = search_list.get(mode as usize).iter().find(|v| has_condition_index(*v, con)) { result.commit_4(entry); }
        if is_dark {
            if let Some(voice) = voice { result.get_sound().voice_id = voice.into(); }
            else { result.get_sound().voice_id = Il2CppString::null(); }
        }
        else {
            let style = BattleStyle::names().get(unit.get_job().get_style().value as usize);
            if let Some(style_con) = get_condition_index(style){
                if let Some(entry) = search_list.get(mode as usize).iter().find(|v| has_condition_index(*v, con) && has_condition_index(*v, style_con )) {
                    result.commit_4(entry);
                }
            }
        }
    }
    pub fn swap_asset_condition_enemy_player_unit(condition_unit: Unit, conditionss: &mut AssetConditions) {
        let person = condition_unit.get_person();
        if PersonPool::is_playable(person) { return; }
        if condition_unit.get_person().hash() == Self::PAST_LUEUR {
            if let Some(hero) = PersonPool::get_replacement(0, false, true){
                AssetFlags::set_person_conditions(condition_unit.get_person(), false);
                AssetFlags::set_person_conditions(hero, true);
            }
        }
        else if let Some(replacement) = PersonPool::get_enemy_player_replacement(person, true){
            AssetFlags::set_person_conditions(condition_unit.get_person(), false);
            AssetFlags::set_person_conditions(replacement, true);
            if replacement.index() == 1 { conditionss.flags.set_gender(Gender { value: DVCVariable::LueurGender.get_value() }); }
            else { conditionss.flags.set_gender(replacement.get_dress_gender()); }
        }
    }
    pub fn get_hero_data() -> Option<&'static PersonalDressData> {
        PersonPool::get_replacement(0, false, true)
            .and_then(|p| get_outfit_data().dress.get_personal_dress_by_person(p, p.get_dress_gender().value == 2))
    }
    pub fn apply_hero_mask_to_hair(result: AssetTable_Result, is_hero: bool, mode: i32) {
        if let Some(hero_data) = Self::get_hero_data() {
            let db = get_outfit_data();
            if is_hero {
                if mode == 2 {
                    if let Some(body) = db.hashes.body.get(&hero_data.ubody){ set_result_dress_body_model(result, mode, body.as_str()); }
                }
                else {
                    if let Some(obody) = db.hashes.get_obody(hero_data.ubody) { set_result_dress_body_model(result, mode, obody); }
                }
                result.commit_8(new_accessory("c_trans", format!("{}Acc_Eff_EmblemAura-01-00", if mode == 1 { "o" } else { "u" })));
            }
            let mut index = 0;
            for i in [4, 5, 6, 7, 0, 1]{
                if hero_data.color[i] != 0 {
                    set_color_by_i32(result, index,hero_data.color[i]);
                    index += 1;
                }
                if index == 2 { break; }
            }
        }
    }
    pub fn is_black_dress(person: PersonData) -> bool { Self::BLACK.contains(&person.hash()) }
    pub fn is_red_eyes(person: PersonData) -> bool { Self::RED_EYES.contains(&person.hash()) }
    pub fn evil_enemy_asset_addition(result: AssetTable_Result, person: PersonData, mode: i32) {
        let hash = person.hash();
        let gender = get_outfit_data().get_dress_gender(result.get_dress_model());
        if let Some(pos) = Self::BLACK.iter().position(|v| *v == hash) {
            match pos {
                0..4 => {   // Dark Dress
                    if PersonPool::get_replacement_index(32, false).unwrap_or(32) == 0 {
                        let mut body = if mode == 2 { "uBody_Drg0A" } else { "oBody_Drg0A" }.to_string();
                        body += if gender.value == 2 { "F_c052" } else { "M_c002" };
                        if mode == 2 { result.set_dress_model(body); } else { result.set_body_model(body); }
                    }
                    if mode == 2 {
                        let mut head_acc = "uAcc_head_Dress556".to_string();
                        if pos == 2 { head_acc += "b"; }
                        result.commit_8(new_asset_table_accessory(head_acc.as_str(), "c_head_loc"));
                        result.replace(AssetTable_Modes::combat());
                    }
                }
                4 => {  // PastLueur
                    let index =  DVCVariables::get_dvc_recruitment_index(0);
                    if index == 32 {
                        if mode == 2 {
                            result.set_head_model("uHead_c556");
                            result.set_dress_model("uBody_Sdp0AF_c556");
                        }
                        else {
                            result.set_head_model("oHair_h556");
                            result.set_body_model("oBody_Sdp0AF_c556");
                        }
                    }
                    else {
                        let mut body = if mode == 2 { "uBody_Drg0A" } else { "oBody_Drg0A" }.to_string();
                        body += if gender.value == 2 { "F_c052" } else { "M_c002" };
                        if mode == 2 { result.set_dress_model(body); } else { result.set_body_model(body); }
                    }
                }
                5 => {  // Hood but with Face
                    if mode == 2 {
                        if gender.value == 2 { result.set_dress_model("uBody_Sdp0AF_c559"); }
                    }
                }
                _ => {}
            }
            if pos == 2 && mode == 2 {
                let mut head_acc = "uAcc_head_Dress556".to_string();
                if pos == 2 { head_acc += "b"; }
                result.commit_8(new_asset_table_accessory(head_acc.as_str(), "c_head_loc"));
            }
        }
        if Self::HOODED.contains(&hash) {
            match (mode == 2, gender.value ==2){
                (true, true) => {   // Hooded
                    result.set_dress_model("uBody_Sdp0AF_c557");
                    result.set_hair_model("uHair_null");
                    for loc in ["c_head_loc", "c_spine1_jnt", "c_spine2_jnt"] { result.commit_8(new_accessory(loc, "null")); }
                    result.replace(AssetTable_Modes::combat());
                }
                (true, false) => {
                    result.set_dress_model("uBody_Dct0AM_c400");
                    for loc in ["c_head_loc", "c_spine1_jnt", "c_spine2_jnt"] { result.commit_8(new_accessory(loc, "null")); }
                    result.replace(AssetTable_Modes::combat());
                }
                (false, true) => { result.set_body_model("oBody_Sdp0AF_c559"); }
                (false, false) => { result.set_body_model("oBody_Dct0AM_c400"); }
            }
        }
    }
}