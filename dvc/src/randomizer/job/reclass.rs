use engage::menu::BasicMenuItemAttribute;
use engage::sortie::SortieSelectionUnitManager;
use engage::unit::Gender;
use crate::utils::get_base_classes;
use super::*;
use engage::menu::menus::class_change::{ChangeJobDataProofTypes, ClassChange, ClassChangeJobMenu, ClassChangeJobMenuConfirmDialog, ClassChangeJobMenuItem};

pub fn class_change_job_menu_item_build_attr(this: &mut ClassChangeJobMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
    if !DVCVariables::random_enabled() { return this.attribute; }
    if DVCConfig::get().debug { return BasicMenuItemAttribute::Enable; }
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
    else if DVCConfig::get().debug {
        this.attribute = BasicMenuItemAttribute::Enable;
        this.job_data.is_default_job = true;
        BasicMenuItemAttribute::Enable
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