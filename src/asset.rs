use unity::prelude::*;
use engage::{
    gamedata::{unit::*, *},
};
use unity::il2cpp::object::Array;

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

#[unity::from_offset("App", "UnitEdit", "IsEnable")]
pub fn unit_edit_is_enable(this: &UnitEdit, method_info: OptionalMethod) -> bool;

//Unlock royal classes if asset table entry is found
pub fn unlock_royal_classes(){
    let list = AssetTable::get_list().unwrap();
    let job_list = JobData::get_list().unwrap();
    for j in 0..job_list.len() {
        let current_job = &job_list[j as usize];  
        let job = current_job.jid.get_string().unwrap();
        let flag = current_job.get_flag();
        if flag.value & 1 == 0 {continue; }    // If not reclassable, skip
        if flag.value & 2 != 0 {continue;} // If already reclassable by everyone, skip
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
                this.is_gender = false;
                return false; 
            }
            else {
                 //Male in male only (with female animations)
                if unit.person.get_flag().value & 32 != 0 { 
                    this.is_gender = false;
                    return false; 
                }
                return result;
            }
        }
    }
    if unit.person.get_flag().value & 32 != 0 && this.job.get_flag().value & 4 != 0 {
        if unit.person.get_gender() == 1 { return result; } 
        else { 
            this.is_gender = false; 
            return false; 
        } // Male Crossdressing in female class: true
          // Female Crossdressing in female class: false
    }
    return result;
}