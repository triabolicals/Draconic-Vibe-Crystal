use engage::app::{BasicMenu_Result, Gender, IBitField32, IJobDataMethods};
use crate::dvc::DVC;
use crate::utils::to_opt;
use super::*;
const DLC_CLASSES: [i32; 17] = [
    30291080, 30291392, -631685449, 1398652429, -1116399479, 1156631427, -290620106, 704942064,
    -149285011, 1848240617, -1241492235, 352107958, 5510888, 1982073595, -1406772370, -974713853,
    -1028496122
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DVCJobItem {
    SingleClass,
    GenderM,
    GenderF,
    UnitJob,
}
impl DVCJobItem {
    pub fn from(i: i32) -> Option<Self> {
        match i {
            0 => Some(DVCJobItem::SingleClass),
            1 => Some(DVCJobItem::GenderM),
            2 => Some(DVCJobItem::GenderF),
            3 => Some(DVCJobItem::UnitJob),
            _ => None,
        }
    }
    pub fn get_next_job_hash(current_job_hash: i32, increase: bool, gender: Gender) -> i32 {
        let job_count = JobData::get_count();
        let dlc = DVCVariables::has_dlc();
        let mut current_index =
            if current_job_hash == 1000 && dlc { 1000 }
            else if (current_job_hash == ENCHANTER || current_job_hash == MAGE_CANNON) && !dlc { 0 }
            else {
                to_opt(JobData::try_get_from_hash(current_job_hash)).map(|j| j.index()).unwrap_or(0)
            };
        if gender.value != 0 && current_index == 1000 { current_index = 0; }
        let cc_flag = if gender.value == 0 { 23 } else if gender.value == 1 { 7 } else { 19 };
        loop {
            if increase {
                if current_index == 1000 { current_index = 0 }
                else if (current_index + 1) == job_count {
                    if dlc && gender.value == 0 { return 1000; }
                    else { current_index = 0; }
                }
                else { current_index += 1; }
            }
            else {
                if current_index == 1000 { current_index = job_count - 1; }
                else if current_index == 0 {
                    if dlc && gender.value == 0  { return 1000; }
                    else { current_index = job_count - 1; }
                }
                else { current_index -= 1; }
            }
            if current_index == 0 { return 0; }
            else if let Some(job) = to_opt(JobData::try_get_2(current_index)).filter(|v|
                (v.is_high() || (v.is_low() && !v.has_high_jobs())))
            {
                let h = job.hash();
                let flag = job.get_flag().m_value();
                if DLC_CLASSES.contains(&h) && !DVCVariables::has_dlc() { continue; }
                if flag == 3 || h == LUEUR_CLASS[1] || h == LUEUR_CLASS[2] || (flag & cc_flag == 3) || (gender.value == 3 && (flag & 3 == 3)) {
                    return h;
                }
            }
        }
    }
    pub fn a_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        let var = self.get_var();
        let init = var.get_setting_value();
        let current = menu_item.get_dvc_menu().current_var_values().get(self as usize);
        let main_menu = !DVCVariables::is_main_menu();
        if main_menu || (!main_menu && self == DVCJobItem::UnitJob) || current == init { BasicMenu_Result::pass() }
        else {
            menu_item.set_m_is_command_icon(false);
            var.set_value(current);
            menu_item.update_text1();
            BasicMenu_Result::se_decide()
        }
    }
    fn get_gender_restriction(self, menuitem: DVCBasicMenuItem) -> Gender {
        match self {
            DVCJobItem::SingleClass => Gender::none(),
            DVCJobItem::GenderM => Gender::male(),
            DVCJobItem::GenderF => Gender::female(),
            DVCJobItem::UnitJob => {
                if menuitem.m_index() == 0 { Gender::other() }
                else { PersonPool::get().playable[menuitem.m_index() as usize].get_person_data().get_dress_gender() }
            }
        }
    }
    fn set_menu_value(self, item: DVCBasicMenuItem, value: i32) -> BasicMenu_Result {
        if self == DVCJobItem::UnitJob {
            if !DVCVariables::is_main_menu() { BasicMenu_Result::pass() }
            else {
                DVC::get().unit_job[item.m_index() as usize] = value;
                item.set_value(value);
                item.update_text1();
                BasicMenu_Result::se_cursor()
            }
        }
        else {
            item.set_m_is_command_icon(item.value() != value && !DVCVariables::is_main_menu());
            item.get_dvc_menu().set_current_value(self.get_var(), value);
            item.set_value(value);
            if DVCVariables::is_main_menu() { self.set_variable_value(value); }
            item.update_text1();
            BasicMenu_Result::se_cursor()
        }
    }
    pub fn update_value(self, item: DVCBasicMenuItem, increase: bool, gender: Gender, hash: i32) -> BasicMenu_Result {
        self.set_menu_value(item, Self::get_next_job_hash(hash, increase, gender))
    }
    pub fn custom_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        if let Some(increase) = DVCBasicMenuItem::get_change(true) {
            self.update_value(menu_item, increase, self.get_gender_restriction(menu_item), menu_item.value())
        }
        else { BasicMenu_Result::pass() }
    }
    pub fn plus_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        if self == DVCJobItem::UnitJob {
            if !DVCVariables::is_main_menu() { BasicMenu_Result::pass() }
            else { self.set_menu_value(menu_item, -1) }
        }
        else {
            let rng = Random_2::get_system();
            let job_count = JobData::get_count();
            let rng_hash = JobData::get_2(rng.get_value_2(job_count)).hash();
            self.update_value(menu_item, rng.get_value_2(2) == 0, self.get_gender_restriction(menu_item), rng_hash)
        }
    }
    pub fn x_call(self, menu_item: DVCBasicMenuItem) -> BasicMenu_Result {
        let v = menu_item.value();
        if let Some(job) = to_opt(JobData::try_get_from_hash(v)) {
            let help = Mess::get(job.get_help());
            GameMessage::create_key_wait(menu_item.get_dvc_menu(), help);
            BasicMenu_Result::se_cursor()
        }
        else { BasicMenu_Result::se_miss() }
    }
    pub fn minus_call(self, item: DVCBasicMenuItem) -> BasicMenu_Result {
        if self == DVCJobItem::UnitJob && !DVCVariables::is_main_menu()  { BasicMenu_Result::pass() }
        else { self.set_menu_value(item, 0) }
    }
    pub fn get_var(self) -> DVCVariable {
        match self {
            DVCJobItem::GenderM => DVCVariable::GenderClassM,
            DVCJobItem::GenderF => DVCVariable::GenderClassF,
            _ => DVCVariable::SingleJob,
        }
    }
    pub fn set_variable_value(self, v: i32) { if DVCVariables::is_main_menu() { DVCConfig::get().set_value(self.get_var(), v); } }
    pub fn build_attribute(self) -> BasicMenuItem_Attribute { BasicMenuItem_Attribute::enable() }
}