use engage::{
    sortie::SortieSelectionUnitManager, unit::Gender,
    menu::{BasicMenuItemAttribute, menus::class_change::*}
};
use outfit_core::clamp_value;
use crate::{
    randomizer::{person::{switch_person_reverse}, skill::learn::unit_update_learn_skill},
    utils::{get_base_classes}
};
use super::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ClassTier {
    Base,
    Promoted,
    Special,
}
#[derive(Clone, Copy)]
pub enum ReclassType {
    Enemy,
    Recruitment(bool),
    PlayerSingle(bool),
    PlayerLockout(bool, bool)
}
impl ClassTier {
    pub fn from_job(job: &JobData) -> Self {
        if job.is_high() { Self::Promoted }
        else if job.max_level > 20 || !job.has_high_jobs() { Self::Special }
        else { Self::Base }
    }
}
impl ReclassType {
    pub fn get_from_settings(recruitment: bool) -> Self {
        let mode = DVCVariables::ClassMode.get_value();
        match mode {
            1 => Self::Recruitment(true),
            2 => Self::PlayerSingle(recruitment),
            3|4 => Self::PlayerLockout(recruitment, mode == 4),
            _ => Self::Recruitment(false)
        }
    }
}
fn unit_random_can_reclass(job: &JobData, is_female: bool, high_class: bool, player: bool, emblem: bool) -> bool {
    if !DVCFlags::CustomClass.get_value() { if !JOB_HASH.iter().any(|&hash| hash == job.parent.hash ) { return false;} }
    let hash = job.parent.hash;
    if hash == VILLAGER || ((hash == ENCHANTER || hash == MAGE_CANNON) && !dlc_check()) { false }
    else {
        let job_flags = job.flag.value;
        let flag = if is_female { 51 } else { 39 };  // Ignore + Reclass + GenderExc
        let job_high = job.is_high() || !job.has_high_jobs();
        if let Some(pos) = MONSTER_CLASS.iter().position(|&v| v == hash) {
            player && !emblem && (
                (pos < 2 && DVCVariables::is_main_chapter_complete(11)) ||
                    (pos < 6 && DVCVariables::is_main_chapter_complete(16) && dlc_check())
            )
        }
        else {
            let flag_check = job_flags & flag == 3;     // Only have Reclass
            let tier = (high_class == job_high) || (!high_class == job.is_low());
            flag_check && tier && !job.jid.to_string().contains("_紋章士_")
        }
    }
}
pub fn unit_reclass(unit: &mut Unit, kind: ReclassType) -> bool {
    let current_job = unit.get_job();
    if current_job.jid.to_string().starts_with("JID_紋章士_") { return false; }
    let old_class = current_job.parent.hash;
    let person = unit.get_person();
    let female =
        if unit.person.get_flag().value & 32 != 0 { person.get_gender() == 1 }
        else { unit.get_dress_gender() == Gender::Female };

    let mut tier = ClassTier::from_job(current_job);
    let mut level = unit.get_level();
    let mut il = unit.get_internal_level();
    let mut initial_apt = false;
    match kind {
        ReclassType::Enemy => {
            let high_job = current_job.is_high() || (current_job.get_max_level() > 20 && level > 20);
            let pool: Vec<_> = JobData::get_list().unwrap().iter()
                .filter(|j| unit_random_can_reclass(j, female, high_job, false, false))
                .collect();

            if let Some(j) = pool.get_random_element(Random::get_game()) {
                unit.class_change(j);
                reclass_level_adjustment(unit, level, il, tier);
            }
        }
        ReclassType::Recruitment(random) => {
            if let Some(old_person) = switch_person_reverse(person){
                println!("Recruitment for {} [from {}] CC", unit.person.get_name(), old_person.get_name());
                level = old_person.get_level() as i32;
                il = old_person.get_internal_level() as i32;
                let tier =  ClassTier::from_job(old_person.get_job().unwrap());
                if old_person.get_job().is_some_and(|v| v.is_high()) && il == 0 { il = 20; }
                if random {
                    let high_job = tier == ClassTier::Promoted || level > 20;
                    let pool: Vec<_> = JobData::get_list().unwrap().iter()
                        .filter(|j| unit_random_can_reclass(j, female, high_job, true, false))
                        .collect();

                    if let Some(j) = pool.get_random_element(Random::get_game()) { recruitment_job_level_adjustment(unit, old_person, j, random); }
                    else { recruitment_job_level_adjustment(unit, old_person, unit.person.get_job().unwrap(), random); }
                }
                else { recruitment_job_level_adjustment(unit, old_person, unit.person.get_job().unwrap(), random); }
            }
            adaptive_growths(unit, true);
            initial_apt = true;
        }
        ReclassType::PlayerSingle(recruitment) => {
            if recruitment {
                if let Some(old_person) = switch_person_reverse(person){
                    level = old_person.get_level() as i32;
                    il = old_person.get_internal_level() as i32;
                    if old_person.get_job().is_some_and(|v| v.is_high()) && il == 0 { il = 20; }
                    tier = ClassTier::from_job(old_person.get_job().unwrap());
                }
            }
            if let Some(job) = DVCVariables::get_single_class(tier == ClassTier::Base || level > 20, female){
                unit.class_change(job);
                reclass_level_adjustment(unit, level, il, tier);
            }
            if recruitment {
                adaptive_growths(unit, true);
                initial_apt = true;
            }
        }
        ReclassType::PlayerLockout(recruitment, random) => {
            let playable = lockout::get_all_playable_unit_classes(person);
            let mut selected_job = 0;
            if recruitment {
                if let Some(old_person) = switch_person_reverse(person){
                    level = old_person.get_level() as i32;
                    il = old_person.get_internal_level() as i32;
                    if old_person.get_job().is_some_and(|v| v.is_high()) && il == 0 { il = 20; }
                    tier = ClassTier::from_job(old_person.get_job().unwrap());
                    if !random {
                        if let Some(job) = GameData::get().job_db.get_reclass_job(unit, unit.person.get_job().unwrap(), tier) {
                            if !playable.contains(&job.parent.hash) { selected_job = job.parent.hash; }
                        }
                    }
                }
                initial_apt = true;
            }
            if let Some(job) = JobData::try_get_hash(selected_job){
                unit.class_change(job);
                reclass_level_adjustment(unit, level, il, tier);
            }
            else {
                let is_high = tier == ClassTier::Promoted || (level > 20);
                let pool: Vec<_> = JobData::get_list().unwrap().iter()
                    .filter(|j| unit_random_can_reclass(j, female, is_high, true, false) && !playable.contains(&j.parent.hash))
                    .collect();

                if let Some(j) = pool.get_random_element(Random::get_game()) {
                    unit.class_change(j);
                    reclass_level_adjustment(unit, level, il, tier)
                }
                else {
                    unit.class_change(JobData::try_get_hash(VILLAGER).unwrap());
                    reclass_level_adjustment(unit, level, il, tier)
                }
            }
            if recruitment {
                initial_apt = true;
                adaptive_growths(unit, true); 
            }
        }
    }
    fixed_unit_weapon_mask(unit);
    assign_selected_weapon_mask_by_apt(unit, None);
    update_weapon_apt(unit, initial_apt);
    unit.set_hp(unit.get_capability(0, true));
    unit_update_learn_skill(unit);
    unit.get_job().parent.hash != old_class
}
fn update_weapon_apt(unit: &mut Unit, init: bool) {
    let mut mask = 0;
    if init {
        let rng = Random::get_game();
        if DVCFlags::RandomStartingApt.get_value() {
            unit.original_aptitude.value = 0;
            mask = 1 << (rng.get_value(9)+ 1 );
            if rng.get_value(3) == 0 { mask |= 1 << (rng.get_value(9)+ 1 ); }
        }
        else if unit.job.parent.hash != unit.person.get_job().map(|v| v.parent.hash).unwrap_or(0){
            let mut kinds = vec![];
            kinds.extend(unit.job.weapons.iter().enumerate().filter(|(i, v)| **v >= 1 && *i < 9).map(|(i, _)| i));
            if unit.job.has_high_jobs() {
                unit.job.get_high_jobs().iter()
                    .for_each(|j|{ kinds.extend(j.weapons.iter().enumerate().filter(|(i, v)| **v >= 1 && *i < 9).map(|(i, _)| i)); });
            }
            if let Some(k) = kinds.get_random_element(rng) { mask = 1 << k; }
        }
        if mask == 0 { mask = unit.person.sub_aptitude.value; }
        unit.original_aptitude.value = mask;
    }
    else { mask = unit.original_aptitude.value; }
    for x in 1..9 { if unit.job.weapons[x] == 1 { mask |= 1 << x; } }
    unit.aptitude.value = unit.selected_weapon_mask.value | mask;
}
fn recruitment_job_level_adjustment(unit: &mut Unit, old_person: &PersonData, target: &JobData, random: bool) {
    let old_level = old_person.get_level() as i32;
    let old_il = old_person.get_internal_level() as i32;
    let old_tier = ClassTier::from_job(old_person.get_job().unwrap());
    let new_tier = ClassTier::from_job(target);
    match (old_tier, new_tier) {
        (ClassTier::Base, ClassTier::Promoted)|(ClassTier::Promoted, ClassTier::Base) => {
            if let Some(job) = GameData::get().job_db.get_reclass_job(unit, target, old_tier) {
                println!("New Job for {}: {}", unit.get_name(), job.get_name());
                unit.class_change(job);
                unit.set_hp(unit.get_capability(0, true));
            }
        }
        (ClassTier::Special, ClassTier::Base) => {
            if old_level > 20 {
                if let Some(job) = GameData::get().job_db.get_reclass_job(unit, target, ClassTier::Promoted) {
                    unit.class_change(job);
                }
            }
            else { unit.class_change(target); }
        }
        (ClassTier::Special, ClassTier::Promoted) => {
            if old_level <= 20 {
                if let Some(job) = GameData::get().job_db.get_reclass_job(unit, target, ClassTier::Base) { unit.class_change(job); }
            }
            else { unit.class_change(target); }
        }
        _ => { unit.class_change(target); }
    }
    if random { randomize_selected_weapon_mask(unit, None); }
    else { assign_selected_weapon_mask_by_apt(unit, None); }
    reclass_level_adjustment(unit, old_level, old_il, old_tier);
}
fn reclass_level_adjustment(unit: &Unit, target_level: i32, target_il: i32, from: ClassTier) {
    let total_level = target_level + target_il;
    let job_max_level = unit.get_job().get_max_level() as i32;
    let to = ClassTier::from_job(unit.get_job());
    let mut set_level = target_level;
    if to != from {
        match (from, to) {
            (ClassTier::Special, ClassTier::Base) => {
                if total_level > job_max_level { set_level = clamp_value(total_level - job_max_level, 1, job_max_level); }
            }
            (ClassTier::Special, ClassTier::Promoted) => {
                set_level =
                    if total_level > (job_max_level + 5) { clamp_value(total_level - job_max_level, 1, job_max_level) }
                    else if total_level > (job_max_level - 5) { clamp_value(total_level - job_max_level - 5, 1, job_max_level) }
                    else if total_level > 20 { clamp_value(total_level-20, 1, job_max_level) }
                    else { total_level }
            }
            (ClassTier::Promoted|ClassTier::Base, ClassTier::Special) => { set_level = clamp_value(total_level, 1, job_max_level); }
            _ => {}
        }
    }
    unit.set_level(set_level);
    let max_il = 40 + 10 * GameUserData::get_difficulty(false);
    unit.set_internal_level(clamp_value(total_level-set_level, 0, max_il));
}
pub fn class_change_job_menu_item_build_attr(this: &mut ClassChangeJobMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if !DVCVariables::random_enabled() { return this.attribute; }
    if DVCConfig::get().debug {
        this.job_data.is_enough_item = true;
        return BasicMenuItemAttribute::Enable;
    }
    let job = &this.job_data.job;
    let job_flags = job.flag.value;
    let unit = SortieSelectionUnitManager::get_unit();
    let unit_current_job = &unit.job;
    let job_met = unit.aptitude.value & this.job_data.job_weapon_mask.value;
    let job_wm = this.job_data.job_weapon_mask.value;
    let condition_met =
        if (job_met == job_wm) && (this.job_data.is_enough_level && this.job_data.is_enough_item ) { BasicMenuItemAttribute::Enable }
        else { BasicMenuItemAttribute::Disable };

    if job.parent.hash == unit_current_job.parent.hash {
        this.attribute = condition_met;
        return condition_met;
    }
    if job_flags & 63 == 32 || job_flags & 3 == 0 { return BasicMenuItemAttribute::Hide; }
    let gender = unit.get_dress_gender();
    if (gender == Gender::Male && (job.unit_icon_id_m.is_none() || job_flags & 20 == 4)) || (gender == Gender::Female && (job.unit_icon_id_f.is_none() && job_flags & 20 == 16)) {
        this.attribute = BasicMenuItemAttribute::Hide;
        return BasicMenuItemAttribute::Hide;
    }
    if DVCVariables::Reclassing.get_value() == 2 || DVCVariables::get_single_class(false, gender == Gender::Female).is_some() {    // NoReclassing / Single Class Line
        let hash = job.parent.hash;
        let jid = job.jid.to_string().trim_end_matches("_E").to_string();
        let unit_jid =  unit_current_job.jid.to_string().trim_end_matches("_E").to_string().trim_end_matches("下級").to_string();
        if jid.contains(&unit_jid) || ( jid.contains(&unit_jid) && jid.contains("下級") ) || (jid.contains("メリュジーヌ") && unit_jid.contains("メリュジーヌ")) {
            this.attribute = condition_met;
            this.job_data.is_default_job = true;
            condition_met
        }
        else if unit_current_job.is_high() {
            let lows = get_base_classes(unit_current_job);
            if lows.iter().any(|x| x.parent.hash == hash) || lows.iter().any(|x| x.get_high_jobs().iter().any(|h| h.parent.hash == hash)) {
                this.attribute= condition_met;
                this.job_data.is_default_job = true;
               condition_met
            }
            else { BasicMenuItemAttribute::Hide }
        }
        else if unit_current_job.is_low() && unit_current_job.has_high_jobs() {
            if unit_current_job.get_high_jobs().iter()
                .any(|x| x.parent.hash == hash) || unit_current_job.get_high_jobs().iter()
                .any(|x|  get_base_classes(x).iter().any(|h| h.parent.hash == hash))
            {
                this.attribute = condition_met;
                this.job_data.is_default_job = true;
                condition_met
            }
            else { BasicMenuItemAttribute::Hide }
        }
        else { BasicMenuItemAttribute::Hide }
    }
    else {
        for dlc in ["SID_弾丸装備", "SID_エンチャント"] {
            if job.mask_skills.find_sid(dlc).is_some() && !dlc_check() {
                return BasicMenuItemAttribute::Hide;
            }
        }
        this.attribute = condition_met;
        this.job_data.is_default_job = true;
        condition_met
    }
}
pub fn class_change_a_call_random_cc(item: &ClassChangeJobMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if DVCVariables::ClassMode.get_value() < 3 && DVCVariables::Reclassing.get_value() == 1 && item.attribute == BasicMenuItemAttribute::Enable {
        let unit = ClassChangeJobMenu::get_selected_unit_copy();
        let change_job_list = ClassChange::get_job_list_all();
        let proof = if item.job_data.proof_type > ChangeJobDataProofTypes::Master { 1 } else { 0 };
        unit.aptitude.value = -1;
        let pool: Vec<_> = change_job_list.iter().filter(|&cc_job| can_change_job_to_list(cc_job, unit, proof)).collect();
        if pool.len() > 1 {
            let rng = Random::new(unit.grow_seed as u32);
            if let Some(data) = pool.get_random_element(rng) {
                ClassChangeJobMenuConfirmDialog::create_bind(item.menu, data);
                return BasicMenuResult::se_decide();
            }
        }
    }
    if item.attribute == BasicMenuItemAttribute::Enable {
        ClassChangeJobMenuConfirmDialog::create_bind(item.menu, item.job_data);
        BasicMenuResult::se_decide()
    }
    else { BasicMenuResult::se_miss() }
}