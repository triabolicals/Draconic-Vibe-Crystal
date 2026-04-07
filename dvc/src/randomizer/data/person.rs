use engage::gamedata::{Gamedata, PersonData};
use engage::gamedata::skill::{SkillArray, SkillData};
use engage::gamevariable::GameVariableManager;
use engage::mess::Mess;
use crate::config::DVCVariables;
use crate::enums::PIDS;
use crate::randomizer::person::switch_person;
use crate::utils::get_base_classes;

pub struct PlayableCharacter {
    pub hash: i32,
    pub playable_slot: i32,
    pub skill: i32,
}

impl PlayableCharacter {
    pub fn new(person: &PersonData, slot: i32) -> PlayableCharacter {
        let skill = person.get_common_skills().iter()
            .find(|s| !s.is_hidden())
            .and_then(|s| s.get_skill())
            .map(|skill| skill.parent.hash)
            .unwrap_or(0);

        PlayableCharacter{ hash: person.parent.hash, playable_slot: slot, skill, }
    }
    pub fn update_personal_skill(&self, to_new: bool) {
        if let Some(person) = PersonData::try_get_hash_mut(self.hash) {
            let hash = if to_new { GameVariableManager::get_number(format!("G_P_{}", person.pid).as_str()) } else { self.skill };
            change_personal_skill_hash(person.get_common_skills(), hash);
        }
    }
    pub fn get_person_data(&self) -> &'static PersonData { PersonData::try_get_hash(self.hash).unwrap() }
    pub fn get_person_data_mut(&self) -> &'static mut PersonData { PersonData::try_get_hash_mut(self.hash).unwrap() }
    pub fn get_replacement(&self) -> Option<&'static PersonData> { DVCVariables::get_dvc_person_data(self.playable_slot, false) }
    pub fn get_current_personal(&self) -> Option<&'static SkillData> { 
        self.get_person_data().common_skills.iter().find(|x| !x.is_hidden() ).and_then(|s| s.get_skill())
    }
}

pub struct EnemyCharacter {
    pub hash: i32,
    pub playable_slot: Option<usize>,
}
impl EnemyCharacter {
    pub fn new(person: &PersonData, playable_slot: i32) -> Self {
        let slot = if playable_slot < 41 { Some(playable_slot as usize) } else { None };
        Self { hash: person.parent.hash, playable_slot: slot }
    }
    pub fn get_person_mut(&self) -> &'static mut PersonData { PersonData::try_get_hash_mut(self.hash).unwrap() }
    pub fn update_personal_skill(&self, original_person: &PlayableCharacter) {
        if self.playable_slot.is_none() { return; }
        if let Some(skill) = original_person.get_current_personal(){
            let enemy = self.get_person_mut();
            if let Some(x) = enemy.get_common_skills().list.iter_mut().find(|s| !s.is_hidden()) {
                x.set_index(skill.parent.index);
            }
        }
    }
    pub fn update_person(&self) {
        if PersonData::try_get_hash_mut(self.hash).is_none_or(|p| p.flag.value & 512 != 0) { return; }    // Already updated
        if let Some(src_person) = self.playable_slot.and_then(|x| PersonData::get(PIDS[x])) {
            if let Some(new_person) = switch_person(src_person).or_else(||Some(src_person)){
                let old_job = src_person.get_job().expect(
                    format!("Person #{}: {} does not have a valid class.\nPlease change the JID of Person #{} in Person.xml.",
                            src_person.parent.index, Mess::get_name(src_person.pid), src_person.parent.index).as_str()
                );
                let new_job = new_person.get_job().expect(
                    format!("Person #{}: {} does not have a valid class.\nPlease change the JID of Person #{} in Person.xml.",
                            new_person.parent.index, Mess::get_name(new_person.pid), new_person.parent.index).as_str()
                );
                let jid =
                    if old_job.is_high() && (new_job.is_high() || (new_job.is_low() && new_job.max_level == 40)) { new_job.jid } else if (old_job.is_high() || src_person.get_level() > 20) && (new_job.is_low() && new_job.has_high_jobs()) { new_job.get_high_jobs()[0].jid } else if old_job.is_low() && new_job.is_high() {
                        let lows = get_base_classes(new_job);
                        if lows.len() == 0 { "JID_ソードファイター".into() } else { lows[0].jid }
                    } else { new_job.jid };
                let flag_value =
                    if new_person.pid.to_string() == PIDS[0] {
                        new_person.flag.value | 1536
                    } else { new_person.flag.value | 512 };
                let grow = new_person.get_grow();
                let valid = new_person.unit_icon_id.is_some();
                let icon_name = if valid { new_person.unit_icon_id.unwrap().to_string() } else { "".to_string() };
                let help_name = new_person.help.map(|v| v.to_string()).unwrap_or_else(|| "MPID_H_Unknown".to_string());
                if let Some(person_x) = PersonData::try_get_hash_mut(self.hash) {
                    person_x.hometown = src_person.parent.hash;
                    let job_x = person_x.get_job().expect(
                        format!("Person #{}: {} does not have a valid class.\nPlease change the JID of Person #{} in Person.xml.",
                                person_x.parent.index, Mess::get_name(person_x.pid), person_x.parent.index).as_str()
                    );
                    let level = person_x.get_level();
                    if (job_x.is_low() && job_x.max_level == 20) && new_job.is_high() { person_x.set_level(level); } else if (job_x.is_low() && job_x.max_level == 40) && new_job.is_high() {
                        if level > 20 {
                            person_x.set_level(level - 20);
                            person_x.set_internal_level(20);
                        }
                    } else if job_x.is_high() && new_job.max_level == 40 {
                        let total = if person_x.get_internal_level() == 0 { 20 } else { person_x.get_internal_level() } as u8 + level;
                        person_x.set_level(total);
                        person_x.set_internal_level(0);
                    }
                    person_x.name = new_person.name;
                    person_x.get_flag().value = flag_value;
                    let grow_x = person_x.get_grow();
                    person_x.jid = Some(jid);
                    person_x.set_gender(new_person.get_gender2());
                    person_x.attrs = new_person.attrs;
                    if valid { person_x.unit_icon_id = Some(icon_name.clone().into()); }
                    person_x.help = Some(help_name.into());
                    if !grow_x.is_zero() {
                        for x in 0..11 {
                            if grow_x[x as usize] == 0 { continue; }
                            if grow_x[x as usize] < grow[x as usize] {
                                grow_x[x as usize] = grow[x as usize];
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn change_personal_skill_hash(skill: &mut SkillArray, new_skill_hash: i32) {
    if let Some(new_skill) = SkillData::try_get_hash(new_skill_hash) {
        if let Some(skill_entity) = skill.list.iter_mut().find(|x| !x.is_hidden()){
            skill_entity.set_index(new_skill.parent.index);
        }
    }
}
