use unity::prelude::*;
use engage::{
    gamedata::{unit::*, skill::*, *},
};
use unity::system::List;
use engage::gamedata::item::ItemData;
use unity::il2cpp::object::Array;
use engage::gamevariable::GameVariableManager;
use crate::enums::*;

#[unity::class("App", "AssetTable")]
pub struct AssetTable {
    pub parent: StructBaseFields,
    pub preset_name: Option<&'static Il2CppString>,
    pub mode: i32,
    __: i32,
    pub conditions: Option<&'static Array<&'static Il2CppString>>,
    pub body_model: Option<&'static Il2CppString>,
    pub dress_model: Option<&'static Il2CppString>,
    pub head_model: Option<&'static Il2CppString>,
    pub hair_model: Option<&'static Il2CppString>,
    pub ride_model: Option<&'static Il2CppString>,
    pub ride_dress_model: Option<&'static Il2CppString>,
    pub left_hand: Option<&'static Il2CppString>,
    pub right_hand: Option<&'static Il2CppString>,
    pub trail: Option<&'static Il2CppString>,
    pub magic: Option<&'static Il2CppString>,
    pub body_anim: Option<&'static Il2CppString>, 
    pub ride_anim: Option<&'static Il2CppString>,
    pub info_anim: Option<&'static Il2CppString>,
    pub talk_anim: Option<&'static Il2CppString>,
    pub demo_anim: Option<&'static Il2CppString>,
    pub hub_anim: Option<&'static Il2CppString>,
    pub hair_r: u8,
    pub hair_g: u8,
    pub hair_b: u8,
    pub grad_r: u8,
    pub grad_g: u8,
    pub grad_b: u8,
    pub skin_r: u8,
    pub skin_g: u8,
    pub skin_b: u8,
    pub toon_r: u8,
    pub toon_g: u8,
    pub toon_b: u8,
    pub mask_color_100_r: u8,
    pub mask_color_100_g: u8,
    pub mask_color_100_b: u8,
    pub mask_color_075_r: u8,
    pub mask_color_075_g: u8,
    pub mask_color_075_b: u8,
    pub mask_color_050_r: u8,
    pub mask_color_050_g: u8,
    pub mask_color_050_b: u8,
    pub mask_color_025_r: u8,
    pub mask_color_025_g: u8,
    pub mask_color_025_b: u8,
    other_stuff: [u64; 17],
    pub scale_stuff: [f32; 19], 
    ___: i32,
    pub voice: Option<&'static Il2CppString>,
    pub foot_steps: Option<&'static Il2CppString>,
    pub material: Option<&'static Il2CppString>,
    pub comment: Option<&'static Il2CppString>,
    //ConditionIndexes
}
impl Gamedata for AssetTable {}

#[unity::class("App", "AssetTable.Result")]
pub struct AssetTableResult {
    pub pid: &'static Il2CppString,
    pub jid: &'static Il2CppString,
    pub body_model: &'static Il2CppString,
    pub dress_model: &'static Il2CppString,
    pub head_model: &'static Il2CppString,
    pub hair_model: &'static Il2CppString,
    pub ride_model: &'static Il2CppString,
    pub ride_dress_model: &'static Il2CppString,
    pub left_hand: &'static Il2CppString,
    pub right_hand: &'static Il2CppString,
    pub trail: &'static Il2CppString,
    pub magic: &'static Il2CppString,
    pub body_anim: &'static Il2CppString, 
    pub ride_anim: &'static Il2CppString,
}

#[unity::class("App", "AssetTableAccessory")]
pub struct AssetTableAccessory {
    pub locator: Option<&'static Il2CppString>,
    pub model: Option<&'static Il2CppString>, 
}
//Unlock royal classes if asset table entry is found
pub fn unlock_royal_classes(){
    let list = AssetTable::get_list().unwrap();
    let job_list = JobData::get_list().unwrap();
    for j in 0..job_list.len() {
        let current_job = &job_list[j as usize];  
        let job = current_job.jid.get_string().unwrap();
        let flag = current_job.get_flag();
        if flag.value & 1 == 0 {continue; }    // If not reclassable, skip
        if flag.value & 2 != 0 {continue; } // If already reclassable by everyone, skip
        for x in 0..list.len(){
                //Search all assettable entries
            let asset_entry = &list[x];
            if asset_entry.body_model.is_none() || asset_entry.conditions.is_none() { continue; }
            let mut job_conditions: [bool; 3] = [false; 3];
            let conditions = asset_entry.conditions.unwrap(); 
            for y in 0..conditions.len() {
                if conditions[y].get_string().unwrap() == job { job_conditions[0] = true; }
                if conditions[y].get_string().unwrap() == "女装" { job_conditions[1] = true;}  //Females
                if conditions[y].get_string().unwrap() == "男装" { job_conditions[2] = true;}  // Dudes
            }
            if job_conditions[0] {
                if job_conditions[1] {
                    flag.value = flag.value | 2;
                    flag.value = flag.value | 4;
                }
                else if job_conditions[2] {
                    flag.value = flag.value | 2;
                    flag.value = flag.value | 16;
                }
            }
        }
        // If both Male and Female are flagged, disable flags
        if flag.value & 4 != 0 && flag.value & 16 != 0 {    flag.value = 3; }
    }//エンゲージ技
}
#[unity::class("App", "ClassChange.ChangeJobData")]
pub struct ChangeJobData {
    pub job: &'static JobData,
    pub job_weapon_mask: &'static WeaponMask,
    pub original_job_weapon_mask: &'static WeaponMask,
    pub proof_type: i32, 
    __: i32,
    pub cost_level: &'static Il2CppString,
    pub is_enough_level: bool,
    pub junk: [u8; 7],
    pub cost_weapon_mask: &'static WeaponMask,
    pub equippable_weapon_mask: &'static WeaponMask,
    pub enough_item: bool,
    pub is_gender: bool,
    pub is_default_job: bool,
}

// App.ClassChange.ChangeJobData$$CCCheck hook
#[skyline::hook(offset=0x019c6700)]
pub fn add_job_list_unit(this: &mut ChangeJobData, unit: &Unit, method_info: OptionalMethod) -> bool {
    let result = call_original!(this, unit, method_info);
    // Dancer-lock
    if this.job.jid.get_string().unwrap() == "JID_ダンサー" { 
        if unit.get_job().jid.get_string().unwrap() == "JID_ダンサー" || unit.person.get_job().unwrap().jid.get_string().unwrap() == "JID_ダンサー" {
            if this.job.get_flag().value & 16 != 0 {
                let gender; 
                if unit.edit.is_enabled() { gender = unit.edit.gender; }  // Alear
                else { gender = unit.person.get_gender(); } // Everyone Else 
                if gender == 2 {  
                    this.is_gender = false;
                    return false; 
                }
                this.is_default_job = true;
                return result 
            }
        }
        else {
            this.is_gender = false;
            return false; 
        }
    }
    if this.job.get_flag().value & 16 != 0 {
        let gender = if unit.edit.is_enabled() { unit.edit.gender }  // Alear
                     else { unit.person.get_gender() }; // Everyone Else 
        if gender == 2 {  
            this.is_gender = false;
            return false; 
        }
        else {
            //Male in male only (with female animations)
            if unit.person.get_flag().value & 32 != 0 { 
                this.is_gender = false;
                return false; 
            }
        }
        return result;
    }
    if unit.person.get_flag().value & 32 != 0 && this.job.get_flag().value & 4 != 0 {
        if unit.person.get_gender() == 1 { 
            this.is_gender = true;
            let job_wm = this.job_weapon_mask.value;
            if unit.aptitude.value & job_wm == job_wm && (this.is_enough_level && this.enough_item ) {
                return true;
            }
            return result; 
        } 
        else { 
            this.is_gender = false; 
            return false; 
        } // Male Crossdressing in female class: true
          // Female Crossdressing in female class: false
    }
    return result;
}
#[skyline::from_offset(0x01bb0100)]
pub fn unit_god_get_state(this: &Unit, method_info: OptionalMethod) -> i32;

#[skyline::from_offset(0x1bb2260)]
pub fn get_body_anims(this: &AssetTableResult, method_info: OptionalMethod) -> &'static mut List<Il2CppString>;

#[skyline::from_offset(0x01a21460)]
pub fn get_engage_attack(this: &Unit, method_info: OptionalMethod) -> Option<&'static SkillData>;

#[skyline::from_offset(0x03785820)]
pub fn copy_str(string: &Il2CppString, method_info: OptionalMethod) -> &'static mut Il2CppString;

#[skyline::from_offset(0x01bb2270)]
pub fn asset_table_result_accessory_list(this: &AssetTableResult, method_info: OptionalMethod) -> &'static mut List<AssetTableAccessory>;

#[skyline::from_offset(0x01baf640)]
pub fn try_add_accessory_list(this: &mut List<AssetTableAccessory>, accessory: &AssetTableAccessory, method_info: OptionalMethod);

#[skyline::from_offset(0x01bb5a90)]
pub fn get_for_talk(pid: &Il2CppString, method_info: OptionalMethod) -> &'static mut AssetTableResult;

#[skyline::from_offset(0x01bb2d80)]
pub fn asset_table_result_god_setup(this: &mut AssetTableResult, mode: i32, god_data: &GodData, is_darkness: bool, conditions: &Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult;
// Fixing Engage Attack Animation (kinda)

#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook(this: &mut AssetTableResult, mode: i32, unit: &Unit, equipped: &ItemData, conditions: &Array<&'static Il2CppString>, method_info: OptionalMethod) -> &'static mut AssetTableResult {
    let result = call_original!(this, mode, unit, equipped, conditions, method_info);
    if GameVariableManager::get_number("G_Random_God_Mode") < 2 { return result; }
    unsafe {
        let state = unit_god_get_state(unit, None);
        if state == 2 && unit.force.unwrap().force_type == 0 {
            let engage_attack = get_engage_attack(unit, None);
            if engage_attack.is_some() {
                let list = get_body_anims(result, None);
                let sid = engage_attack.unwrap().sid;
                let mut emblem_index = 50;
                let mut animation_index = 50;
                for x in 0..list.len() {
                    for y in 0..20 {
                        if crate::utils::str_contains(list[x], ENGAGE_PREFIX[y]){
                            animation_index = x;
                            break;
                        }
                    }
                    if animation_index != 50 { break; }
                }
                for x in 0..20 {
                    let e_sid = format!("SID_{}エンゲージ技", EMBLEM_ASSET[x as usize]);
                    if crate::utils::str_contains(sid, &e_sid) {
                        emblem_index = x; 
                        break;
                    }
                    if x == 12 && crate::utils::str_contains(sid, "SID_三級長エンゲージ") {
                        emblem_index = x;   // Three Houses Engage Attack
                        break;
                    }
                    if crate::utils::str_contains(sid, "SID_三級長エンゲージ技＋") {
                        emblem_index = 20;
                        break;
                    }
                }
                if animation_index == 50 || emblem_index == 50 { return result; }
                let mut new_engage_animation: &Il2CppString = "".into();
                let gender_str;
                let gender; 
                if unit.edit.is_enabled() { gender = unit.edit.gender; }  // Alear
                else { gender = unit.person.get_gender(); } // Everyone Else 
                if gender == 2 {  gender_str = "F"; }
                else {
                    if unit.person.get_flag().value & 32 != 0 { 
                        gender_str = "F";
                    }
                    else { gender_str = "M"; }
                }   
                    // 0    1         2       3     4      5     6      7      8     9       10     11      12      13    14    15     16       17    18    19
                // &[ "Mar", "Sig", "Cel", "Mic", "Roy", "Lei", "Luc", "Lyn", "Ike", "Byl", "Cor", "Eir", "Thr", "Tik", "Hec", "Ver", "Sor", "Cmi", "Chr", "Ler" ];
                let mpid = unit.person.get_name().unwrap().get_string().unwrap();
                result.ride_model = "".into();
                result.ride_dress_model = "".into();
                match emblem_index {    //Marth, Roy, Leif, Lucina, Ike, Byleth, Dragon Blast
                    0|4|5|6|8|9|19 => { new_engage_animation = format!("{}1A{}-Sw1_c000_N", ENGAGE_PREFIX[ emblem_index as usize], gender_str).into(); }
                    2|15|16 => { new_engage_animation = format!("{}1A{}-Mg1_c000_N", ENGAGE_PREFIX[ emblem_index as usize], gender_str).into(); } //Celica / Veronica / Soren 
                    1 => {  //Sigurd
                        result.ride_model = "uRig_HorsR".into();
                        result.ride_dress_model = "uBody_Sig0BR_c531".into();
                        new_engage_animation = format!("Sig1B{}-Sw1_c000_N", gender_str).into();
                    }
                    3 => {  //Micaiah
                        if ( mpid == "MPID_Lueur" && gender == 1 ) || mpid == "MPID_Boucheron" || mpid == "MPID_Bonet" || mpid == "MPID_Vandre" || mpid == "MPID_Mauve" {
                            new_engage_animation = "Mic1AM-Mg1_c001_N".into();
                        }
                        else if mpid == "MPID_Jean" || mpid == "MPID_Staluke" || mpid == "MPID_Clan" { new_engage_animation = "Mic1AM-Mg1_c501_N".into(); }
                        else if mpid == "MPID_Saphir" { new_engage_animation = "Mic1AF-Mg1_c254_N".into(); }
                        else { new_engage_animation = format!("Mic1A{}-Sw1_c000_N", gender_str).into(); }
                    }
                    7 => { new_engage_animation = format!("Lyn1A{}-Bw1_c000_L", gender_str).into(); }   // Lyn 
                    10 => { new_engage_animation = format!("Cor1A{}-Ft1_c000_N", gender_str).into(); }  // Corrin
                    11 => {
                        if mpid == "MPID_Jean" { new_engage_animation = "Eir1AM-Sw1_c103_N".into(); }
                        else if mpid == "MPID_Anna" { new_engage_animation = "Eir1AF-Sw1_c552_N".into(); }
                        else { new_engage_animation = format!("Eir1A{}-Sw1_c000_N", gender_str).into(); }
                    }   // Eirika
                    12 => { new_engage_animation = format!("Thr1A{}-Ax1_c000_N", gender_str).into(); } //Houses Unite
                    13 => { new_engage_animation = "Tik1AT-Mg1_c000_M".into(); }    //Tiki
                    14 => { new_engage_animation = format!("Hec1A{}-Ax1_c000_N", gender_str).into(); } // Hector
                    17 => { //Camilla 
                        result.ride_model = "uRig_DragR".into();
                        result.ride_dress_model = "uBody_Cmi0DR_c561".into(); 
                        new_engage_animation = format!("Cmi1D{}-Ax1_c000_N", gender_str).into();
                    }
                    18 => {     //Chrom
                        if mpid == "MPID_Jean" {  new_engage_animation = "Chr1AM-Sw1_c103_N".into(); }
                        else if mpid == "MPID_Mauve" || mpid == "MPID_Boucheron" { new_engage_animation = "Chr1AM-Sw1_c502_N".into(); }
                        else if mpid == "MPID_Anna" { new_engage_animation = "Chr1AF-Sw1_c552_N".into(); }
                        else if ( mpid == "MPID_Lueur" && gender == 2 )  || mpid == "MPID_Chloe" || mpid == "MPID_Jade" || mpid == "MPID_Ivy" || mpid == "MPID_Merin" || mpid == "MPID_Saphir" {
                            new_engage_animation = "Chr1AF-Sw1_c254_N".into();
                        }
                        else if mpid == "MPID_El" || mpid == "MPID_Selestia" { new_engage_animation = "Chr1AF-Sw1_c254_N".into(); }
                        else {
                            new_engage_animation = format!("Chr1A{}-Sw1_c000_N", gender_str).into();
                        }
                    }
                    20 => { new_engage_animation = format!("Thr2A{}-Ax1_c000_N", gender_str).into(); }  // Houses Unite+
                    _ => { return result; }

                }
                let accessory_list = asset_table_result_accessory_list(result, None);
                if emblem_index != 15 { // Removing the Summoning Wall if it exists
                    for x in 0..accessory_list.len() {
                        if accessory_list[x].model.is_some() {
                            let model = accessory_list[x].model.unwrap().get_string().unwrap();
                            if model == "uAcc_Event_SummonStone" || model == "uAcc_Event_SummonStoneB" {
                                accessory_list[x].model = None;
                                accessory_list[x].locator = None;
                            }
                        }
                    }
                }
                else {
                    let accessory_class = Il2CppClass::from_name("App", "AssetTable").unwrap().get_nested_types()[2];
                    let new_accessory = Il2CppObject::<AssetTableAccessory>::from_class( accessory_class ).unwrap();
                    new_accessory.model = Some("uAcc_Event_SummonStoneB".into() );
                    new_accessory.locator = Some("reserve4_loc".into());
                    try_add_accessory_list(accessory_list, new_accessory, None);
                }
                list[animation_index as usize] = copy_str(new_engage_animation, None);
            }
        }
        if crate::utils::str_contains(unit.person.pid, "PID_M022_紋章士") {
            let pid = unit.person.pid.get_string().unwrap();
            for x in EMBLEM_ASSET {
                let pid2 = format!("PID_M022_紋章士_{}", x);
                if pid == pid2 {
                    let variable = format!("G_R_GID_{}", x);
                    let gid = GameVariableManager::get_string(&format!("G_R_GID_{}", x)).get_string().unwrap();
                    for y in 12..19 {
                        if gid == format!("GID_{}", EMBLEM_ASSET[y]) {
                            let god = GodData::get(&format!("GID_E006_敵{}", EMBLEM_ASSET[y])).unwrap();
                            return asset_table_result_god_setup(this, mode, god, true, conditions, method_info);
                        }
                    }
                    let god = GodData::get(&gid).unwrap();
                    return asset_table_result_god_setup(this, mode, god, true, conditions, method_info);
                }
            }
        }
    result
    }
}