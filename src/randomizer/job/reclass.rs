use super::*;

#[skyline::hook(offset=0x019c6700)]
pub fn add_job_list_unit(this: &mut ChangeJobData, unit: &Unit, method_info: OptionalMethod) -> bool {
    let result = call_original!(this, unit, method_info);
    if !DVCVariables::random_enabled() { return result; }
    if CONFIG.lock().unwrap().debug {
        //this.is_gender = true;
        this.is_default_job = true;
        return true;
    }
    // Dancer-lock
    if this.job.mask_skills.find_sid("SID_踊り").is_some() { 
        if unit.get_job().mask_skills.find_sid("SID_踊り").is_some() || unit.person.get_job().unwrap().mask_skills.find_sid("SID_踊り").is_some() {
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
        let gender = if unit.edit.is_enabled() { unit.edit.gender } else { unit.person.get_gender() };
        if gender == 2 {  
            this.is_gender = false;
            return false; 
        }
        else {
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
        }
    }
    return result;
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


#[unity::class("App", "ClassChangeJobMenuItem")]
pub struct ClassChangeJobMenuItem {
    pub menu: u64,
    pub junk: [u8; 0x4c],
    pub job_data: &'static mut ChangeJobData,
    pub atr: i32,
}
pub fn class_change_a_call_random_cc(item: &ClassChangeJobMenuItem, _method_info: OptionalMethod) -> i32 {
    if item.atr != 1 { return 0x800; }
    if !GameVariableManager::get_bool(DVCVariables::RECLASS_KEY) || !DVCVariables::random_enabled() {
        if item.atr == 1 {
            unsafe { class_change_confirm_bind(item.menu, item.job_data, None); }
            return 0x80;
        }
        return 0x800;
    }
    else {
        let unit = unsafe { class_change_get_unit(None) };
        let change_job_list = unsafe { get_job_list(None) };
        // for CCCheck to get classes.
        let proof = if item.job_data.proof_type > 0 { 1 } else { 0 };
        unit.aptitude.value = -1; 
        let pool: Vec<_> = change_job_list.iter().filter(|&cc_job|{
            if unsafe { cc_check(cc_job, unit, None) } { add_to_list(cc_job, unit, proof) }
            else { false } 
           }).collect();
        let rng = Random::instantiate().unwrap();
        let seed = unit.grow_seed; 
        rng.initialize(seed as u32);
        if pool.len() > 1 { 
            let pool_size = pool.len() as i32;
            let class = rng.get_value(pool_size) as usize;
            unsafe { class_change_confirm_bind(item.menu, pool[class], None); }
        }
        else { unsafe { class_change_confirm_bind(item.menu, item.job_data, None); } }
    }
    return 0x80;
}
pub fn black_list_jobs() {
    JOB_BLACK_LIST.lock().unwrap().iter()
        .flat_map(|&i| JobData::try_index_get_mut(i))
        .for_each(|job| { 
            let flag = job.get_flag();
            flag.value &= !2;
            flag.value |= 32;
        }
    )
}

#[skyline::from_offset(0x019c76c0)]
fn class_change_confirm_bind(this: u64, data: &ChangeJobData, method_info: OptionalMethod);

#[skyline::from_offset(0x01ea4680)]
pub fn class_change_get_unit(method_info: OptionalMethod) -> &'static mut Unit;
