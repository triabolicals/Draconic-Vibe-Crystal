use super::*;
use animation::MONSTERS;
use engage::gamedata::{*, item::*};

pub const MONSTER_PERSONS: [&str; 8] = ["PID_G000_幻影飛竜", "PID_E004_異形兵_異形飛竜", "PID_G000_幻影狼", "PID_E001_異形兵_異形狼", "PID_E006_Boss", "PID_S006_幻影竜", "PID_M019_異形竜", "PID_M026_ソンブル_竜型"];
pub const SCALE: [f32; 8] = [  1.0, 1.0, 1.0, 1.0, 0.40, 1.0, 1.0, 0.40];
pub static mut OLD_EMBLEM: String = String::new();

#[unity::class("App", "BattleInfoSide")]
pub struct BattleInfoSide {
    info: u64,
    pub side_type: i32,
    pub unit: Option<&'static Unit>,
    pub item: &'static UnitItem,
}

#[unity::class("Combat", "CharacterGameStatus")]
pub struct CharacterGameStatus {
    pub appearance: u64,
    pub emblem_ident: Option<&'static Il2CppString>,
    pub side: i32,
    pub stun: bool,
    pub unit: Option<&'static Unit>,
    pub person: &'static mut PersonData,
    pub job_data: &'static JobData,
    pub force: &'static Force,
    pub name: &'static Il2CppString,
    pub values: [i32; 9],
    pub weapon: Option<&'static mut UnitItem>,
}

#[unity::class("Combat", "CombatRecord")]
pub struct CombatRecord {
    pub is_enemy_attack: bool,
    pub combat_style: i32,
    calc: u64,
    sim_calc: u64,
    pub game_status: &'static mut Array<&'static mut CharacterGameStatus>,
    pub chain_atk: &'static mut Array<&'static mut CharacterGameStatus>,
    pub dragonize: &'static mut Array<&'static mut CharacterGameStatus>,
    junk: [u64; 3],
    pub map_distance: i32,
}

pub struct CombatRecordDisplayClass85 {
    pub this: &'static CombatRecord,
    pub calc: u64,
}
pub fn is_monster_class(unit: &Unit) -> bool {
    let jid = unit.get_job().jid.to_string();
    if unit.person.get_bmap_size() > 1 { false }
    else { super::animation::MONSTERS.iter().any(|&monster| monster == jid)  }

}
pub fn is_emblem_class(unit: &Unit) -> bool {
    let pid = unit.person.pid.to_string();
    let jid = unit.get_job().jid.to_string();
    let hash = unit.get_job().parent.hash;
    if !crate::randomizer::job::JOB_HASH.iter().any(|&h| h == hash) { return false; }
    if unit.person.get_bmap_size() > 1  { return false; }
    if let Some(pos1) = EMBLEM_ASSET.iter().position(|&x1| format!("JID_紋章士_{}", x1) == jid) {
        if pos1 == 23 || pos1 == 19 { return false; }   // No Lueur / Ephiram
        if let Some(pos2) = EMBLEM_ASSET.iter().position(|&x2| pid.contains(x2)) { return pos1 != pos2;  }
        else { return true; }
    }
    return false;
}
fn is_transfrom(unit: Option<&Unit>) -> bool{
    if let Some(u) = unit { is_monster_class(u) | is_emblem_class(u) }
    else { false }
}
#[skyline::hook(offset=0x029285f0)]
pub fn change_dragon(this: &mut CombatRecord, calc_side: i32, param_3: &CombatRecordDisplayClass85, method_info: OptionalMethod) {
    call_original!(this, calc_side, param_3, method_info);
    let side = unsafe { combat_side_convert_from(calc_side, this.is_enemy_attack,  None)};
    if this.game_status[side as usize].unit.is_some() {

        let state = unsafe { unit_god_get_state(this.game_status[side as usize].unit.unwrap(), None) };
        let status = this.game_status[side as usize].unit.unwrap().status.value;
        let gender =  this.game_status[side as usize].person.gender;
        let transform = is_transfrom(this.game_status[side as usize].unit);
        println!("Transform for {}: {} Side: {}", Mess::get_name(this.game_status[side as usize].unit.unwrap().person.pid), transform, side);
        if state >= 2 { return; }   //Engage Attack
        if this.game_status[side as usize].weapon.is_some() {

            let item = this.game_status[side as usize].weapon.as_ref().unwrap().item.parent.index;
            let item_data = ItemData::try_index_get(item).unwrap();
            println!("Transform Item Check: {}", Mess::get(item_data.name));
            let kind =  this.game_status[side as usize].weapon.as_ref().unwrap().item.kind;
            if this.game_status[side as usize].weapon.as_ref().unwrap().item.iid.contains("チキ_") {
                println!("Tiki Check");
                if state != 0 { return; } 
                if this.game_status[side as usize].person.pid.contains("_チキ") { return; }
                this.combat_style |= 1 << 22;
                let person_index = this.game_status[side as usize].person.parent.index;
                let item_mut = ItemData::try_index_get_mut(item).unwrap();
                println!("Tiki Transform Person Replacement");
                if item_mut.flag.value & 67108864 == 0 {
                    let flag = item_mut.flag.value;
                    item_mut.flag.value |= 67108864;
                    this.game_status[side as usize].person = PersonData::get_mut("PID_闘技場_チキ").unwrap();
                    call_original!(this, calc_side, param_3, method_info);
                    item_mut.flag.value = flag;
                    this.game_status[side as usize].person = PersonData::try_index_get_mut(person_index).unwrap();
                    return;
                }
                else {
                    this.game_status[side as usize].person = PersonData::get_mut("PID_闘技場_チキ").unwrap();
                    call_original!(this, calc_side, param_3, method_info);
                    this.game_status[side as usize].person = PersonData::try_index_get_mut(person_index).unwrap();
                    return; 
                }
            }
            if state != 0 && kind != 9 { return; }
            println!("Dragon State: {}, status: {}, transform: {}", state,status, transform);
            if gender > 0 && transform {
                if item > 2 {
                    this.combat_style |= 1 << 22;
                    let item_mut = ItemData::try_index_get_mut(item).unwrap();
                    if item_mut.flag.value & 67108864 == 0 {
                        let flag = item_mut.flag.value;
                        item_mut.flag.value |= 67108864;
                        call_original!(this, calc_side, param_3, method_info);
                        item_mut.flag.value = flag;
                        return;
                    }
                }
            }
        }
    }
/* 

    if let Some(unit_item) = &mut this.game_status[side as usize].weapon {
        let item = unit_item.item.parent.index;
        if unit_item.item.iid.contains("IID_チキ_") {   // Tiki Weapon
            if let Some(unit) = &mthis.game_status[side as usize].unit {
                this.combat_style |= (1 << 22);
                if unit.person.pid.contains("_チキ") { return; }    // Not Tiki
                if  unsafe { unit_god_get_state(unit, None) } != 0 { return; }   // Tiki Engage
                let item_mut = ItemData::try_index_get_mut(item).unwrap();
                let person_index = this.game_status[side as usize].person.parent.index;
                if item_mut.flag.value & 67108864 == 0 {
                    let flag = item_mut.flag.value;
                    item_mut.get_flag().value |= 67108864;
                    this.game_status[side as usize].person = PersonData::get_mut("PID_闘技場_チキ").unwrap();
                    call_original!(this, calc_side, param_3, method_info);
                    item_mut.get_flag().value = flag;
                    this.game_status[side as usize].person = PersonData::try_index_get_mut(person_index).unwrap();
                    return;
                }
                else {

                }
            }
        }
        if let Some(unit) = this.game_status[side as usize].unit {
            let state = unsafe { unit_god_get_state(unit, None) };
            if state != 0 && unit_item.item.kind != 9 { return; }
            if unit.person.gender > 0 && (is_monster_class(unit) || is_emblem_class(unit)) {
                this.combat_style |= (1 << 22);
                if item > 2 {
                    let item_mut = ItemData::try_index_get_mut(item).unwrap();
                    if item_mut.flag.value & 67108864 == 0 {
                        let flag = item_mut.flag.value;
                        item_mut.get_flag().value |= 67108864;
                        call_original!(this, calc_side, param_3, method_info);
                        item_mut.get_flag().value = flag;
                        return;
                    }
                }
            }
        }
    }
    call_original!(this, calc_side, param_3, method_info);
    */
}
#[skyline::from_offset(0x0247ce20)]
fn combat_side_convert_from(side_type: i32, is_reversed: bool, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x027def50)]
fn character_game_status_is_valid(gs: &CharacterGameStatus, method_info: OptionalMethod) -> bool;

#[skyline::hook(offset=0x027e0880)]
pub fn combat_character_game_status_import(this:&mut CharacterGameStatus, side: i32, calc: u64, side_type: i32, distance: i32, method_info: OptionalMethod) {
    call_original!(this, side, calc, side_type, distance, method_info);
    if this.emblem_ident.is_some() && this.unit.is_some() {
        let unit = this.unit.unwrap();
        if unsafe { unit_god_get_state(unit, None) } > 1 {
            /*
            if let Some(pos) = ENGAGE_PREFIX.iter().position(|&engage| emblem == engage) {
                let old_emblem = 
                    match pos {
                        20|21|22 => { ENGAGE_PREFIX[9]},
                        23 => { ENGAGE_PREFIX[11] },
                        13 => { ENGAGE_PREFIX[0] },
                        14 => { ENGAGE_PREFIX[7] },
                        18 => { ENGAGE_PREFIX[6] },
                        17 => { "Ike"},
                        16 => { "Cor"},
                         _ => { ENGAGE_PREFIX[pos] },
                    };
                unsafe { OLD_EMBLEM = pos.to_string() };
            }
            else if emblem == "Sen" { unsafe { OLD_EMBLEM = "Ike".to_string() }; }
            else if emblem == "Cam" { unsafe { OLD_EMBLEM = "Cor".to_string() }; }
            else { unsafe { OLD_EMBLEM = "-".to_string() };  }
            */
            if let Some(engage_attack) =  unsafe { get_engage_attack(unit, None) } {
                let sid = engage_attack.sid.to_string();
                let emblem_index = if let Some(pos) = EMBLEM_ASSET.iter().position(|god| sid.contains(god)) { pos }
                    else if sid.contains("三級長エンゲージ技＋") { 20 }
                    else if sid.contains("三級長エンゲージ") { 12 }
                    else { 50 };
                if emblem_index > 20 { return; }
                let new_emblem_id = 
                    match emblem_index {
                        12|20 => { "Ede" },
                        16 => { "Sen"},
                        17 => { "Cam"},
                        _ => { ENGAGE_PREFIX[emblem_index] }
                    };
                this.emblem_ident = Some(new_emblem_id.into());
            }
        }
    }
}

#[skyline::hook(offset=0x01bb4290)]
pub fn asset_table_result_setup_person_hook(this: &AssetTableResult, mode: i32, 
    person: Option<&PersonData>, job: Option<&JobData>, equipped: Option<&ItemData>, conditions: &Array<&Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult 
{

    match (person, job) {
        (Some(dragon), Some(dragon_job)) => {
            // println!("Attempting to transform {} in {}", dragon.get_name().unwrap(), dragon_job.name);
            let jid = dragon_job.jid.to_string();
            if dragon.pid.contains("_竜化")  {
                if let Some(item) = equipped {
                    if let Some(emblem) = EMBLEM_ASSET.iter().find(|&x1| jid.contains(x1)) {
                        let pid = format!("PID_闘技場_{}", emblem);
                        let emblem_person = PersonData::get(pid);
                        let result = call_original!(this, mode, emblem_person, emblem_person.unwrap().get_job(), equipped, conditions, method_info);
                        return result;
                    }
                    if let Some(pos) = MONSTERS.iter().position(|&x| x == jid) {
                        let monster_person = PersonData::get(MONSTER_PERSONS[pos]);
                        let result = call_original!(this, mode, monster_person, job, equipped, conditions, method_info);
                        result.scale_stuff[0] = SCALE[pos];
                        return result;
                    }
                    if item.flag.value & 67108864 != 0 {
                        let person = get_dragon_stone_actor(item, dragon_job);
                        let result = call_original!(this, mode, person.0, job, equipped, conditions, method_info);
                        result.scale_stuff[0] = person.1;
                        return result;
                    }
                }
            }
        },
        (None, Some(job1)) => {  // 
            // println!("Attempting to transform in {}", job1.name);
            let jid = job1.jid.to_string();
            if let Some(pos) = MONSTERS.iter().position(|&x| x == jid) {
                let monster_person = PersonData::get(MONSTER_PERSONS[pos]);
                let result = call_original!(this, mode, monster_person, job, equipped, conditions, method_info);
                result.scale_stuff[0] = SCALE[pos];
                return result;
            }
            if let Some(emblem) = EMBLEM_ASSET.iter().find(|&x1| jid.contains(x1)) {
                println!("Found Emblem");
                let pid = format!("PID_闘技場_{}", emblem);
                let emblem_person = PersonData::get(pid);
                let result = call_original!(this, mode, emblem_person, emblem_person.unwrap().get_job(), equipped, conditions, method_info);
                return result;
            }
            if let Some(item) = equipped {
                if item.flag.value & 67108864 != 0 {
                    let person = get_dragon_stone_actor(item, job1);
                    let result = call_original!(this, mode, person.0, job, equipped, conditions, method_info);
                    result.scale_stuff[0] = person.1;
                    return result;
                }
            }
        }
        _ => {},
    }
    call_original!(this, mode, person, job, equipped, conditions, method_info)
}

pub fn get_dragon_stone_actor(item: &ItemData, job: &JobData) -> (Option<&'static PersonData>, f32) {
    if str_contains(item.iid, "IID_チキ") && item.flag.value & 128 == 0 { return (PersonData::get("PID_E001_Boss_竜化"), 1.0); }    //Tiki
    let i_item = item.iid.to_string();

    match i_item.as_str() {
        "IID_氷のブレス"|"IID_氷塊" => { return (PersonData::get("PID_遭遇戦_異形飛竜"), 1.0); },   //Corrupted Wyvern
        "IID_炎塊"|"IID_火のブレス" => { return (PersonData::get("PID_M011_異形竜"), 1.0); }  //Corrupted Wyrm
        _ => {},
    }
    let jid = job.jid.to_string();
    if jid == "JID_裏邪竜ノ娘" { (PersonData::get("PID_エル_竜化"), 0.4) }
    else if jid ==  "JID_裏邪竜ノ子" { (PersonData::get("PID_ラファール_竜化"), 0.4) }
    else { (None, 0.0) }
}
