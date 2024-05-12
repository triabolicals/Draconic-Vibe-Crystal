use unity::prelude::*;
use engage::{
    menu::{BasicMenuResult, config::{ConfigBasicMenuItemSwitchMethods, ConfigBasicMenuItem}},
    gamevariable::*,
    gameuserdata::*,
    force::*,
    mess::*,
    gamedata::{unit::*, dispos::ChapterData, person::*, job::*, *},
};
use unity::system::List;
use super::CONFIG;
use crate::utils::*;
use unity::il2cpp::object::Array;

#[unity::class("App", "AssetTable")]
pub struct AssetTable {}
impl Gamedata for AssetTable {}

#[skyline::from_offset(0x0211d240)]
pub fn get_condition_names(method_info: OptionalMethod) -> &'static List<Il2CppString>;

#[unity::from_offset("App","AssetTable", "get_Conditions")]
pub fn assettable_get_conditions(this: &AssetTable, method_info: OptionalMethod) -> &'static Array<&'static Il2CppString>;

#[unity::from_offset("App","AssetTable", "get_BodyAnim")]
pub fn assettable_get_bodyanim(this: &AssetTable, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App","AssetTable", "get_BodyModel")]
pub fn assettable_get_bodymodel(this: &AssetTable, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[unity::from_offset("App", "UnitEdit", "IsEnable")]
pub fn unit_edit_is_enable(this: &UnitEdit, method_info: OptionalMethod) -> bool;

use std::fs::File;
use std::sync::Mutex;
use std::io::Write;

pub fn print_asset_table_stuff(){
    let filename = format!("sd:/Draconic Vibe Crystal/AssetTable_Condition_Names.txt");
    let mut f = File::options().create(true).write(true).truncate(true).open(filename).unwrap();
    unsafe {
        let list = AssetTable::get_list().unwrap();
        for x in 0..list.len(){
            let asset_entry = &list[x];
            let conditions = assettable_get_conditions(asset_entry, None);
            let mut condition_string = "".into();
            for y in 0..conditions.len() {
                condition_string = format!("{}\t{}", condition_string, conditions[y].get_string().unwrap());
            }
            let body_act = assettable_get_bodyanim(asset_entry, None);
            if body_act.is_some() {
                writeln!(&mut f, "{} - Conditions: {}\nBody Animation: {}", x, condition_string, body_act.unwrap().get_string().unwrap());
            }
        }
    }
}

//Unlock royal classes if asset table entry is found
pub fn unlock_royal_classes(){
    let royals = ["JID_邪竜ノ娘", "JID_アヴニール下級", "JID_アヴニール", "JID_フロラージュ下級", "JID_フロラージュ", "JID_スュクセサール下級", "JID_スュクセサール", "JID_ティラユール下級", "JID_ティラユール", "JID_リンドブルム下級", "JID_リンドブルム", "JID_スレイプニル下級", "JID_スレイプニル", "JID_ピッチフォーク下級", "JID_ピッチフォーク", "JID_クピードー下級", "JID_クピードー", "JID_ダンサー"];
    unsafe {
        let list = AssetTable::get_list().unwrap();
        for job in royals {  
            let current_job = JobData::get(job).unwrap();
            let flag = current_job.get_flag();
            for x in 0..list.len(){
                let asset_entry = &list[x];
                if assettable_get_bodymodel(asset_entry, None).is_none() { continue; }
                let mut job_conditions: [bool; 3] = [false; 3];
                let conditions = assettable_get_conditions(asset_entry, None);
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
        }
    }
}
#[unity::class("App", "ClassChange.ChangeJobData")]
pub struct ChangeJobData {
    pub job: &'static JobData,
    junk: [u8; 0x38],
    pub enough_item: bool,
    pub is_gender: bool,
    pub is_default_job: bool,
}

// App.ClassChange.ChangeJobData$$CCCheck hook
#[skyline::hook(offset=0x019c6700)]
pub fn add_job_list_unit(this: &mut ChangeJobData, unit: &Unit, method_info: OptionalMethod) -> bool {
    let result = call_original!(this, unit, method_info);
    if this.job.get_flag().value & 16 != 0 {
        unsafe {
            let gender; 
            if unit_edit_is_enable(unit.edit, None) { gender = unit.edit.gender; }  // Alear
            else { gender = unit.person.get_gender(); } // Everyone Else 
            if gender == 2 { 
                println!("Rejected Male Class {}", Mess::get(this.job.name).get_string().unwrap());
                this.is_gender = true;
                return false;
            }
        }
    }
    return result;
}