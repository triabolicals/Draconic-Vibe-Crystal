use engage::gamedata::GodData;
use crate::config::DVCVariables;
use crate::enums::{EMBLEM_ASSET, EMBLEM_GIDS};
use crate::randomizer::data::EmblemData;

pub struct EnemyEmblemData {
    pub emblem_data: EmblemData,
    pub emblem_index: usize,
    pub syncho_stats: [i8; 11],
}

impl EnemyEmblemData {
    pub fn new(god: &GodData) -> Self {
        let gid = god.gid.to_string().trim_start_matches("GID_").to_string();
        let mut syncho_stats = [0; 11];
        for x in 0..11 { syncho_stats[x] = god.syncho_enhance[x]; }
        let emblem_index = EMBLEM_ASSET.iter().position(|x| gid.contains(*x)).unwrap_or(100);
        Self { emblem_data: EmblemData::new(god), syncho_stats, emblem_index}
    }
    pub fn get_replacement_index(&self) -> usize {
        if self.emblem_index < EMBLEM_GIDS.len() { DVCVariables::get_dvc_emblem_index(self.emblem_index as i32, false) } else { 100 }
    }
    pub fn get_replacement_source(&self) -> Option<&'static mut GodData> {
        if self.emblem_index == 100 { None }
        else { DVCVariables::get_god_from_index(self.emblem_index as i32, true) }
    }
    pub fn get_original_god(&self) -> Option<&'static mut GodData> {
        DVCVariables::get_god_from_index(self.emblem_index as i32, false)
    }
    pub fn reset(&self) {
        self.emblem_data.reset_weapons();
        self.emblem_data.reset_all_skills();
        let enhance = &mut self.emblem_data.get_god_mut().syncho_enhance;
        for x in 0..11 { enhance[x] = self.syncho_stats[x]; }
    }
    pub fn update_engage_atk(&self) {
        let enemy = self.emblem_data.get_god_mut();
        if enemy.engage_attack.is_some() {
            if let Some(replacement) = self.get_replacement_source() {
                let src_engage = replacement.get_engage_attack().map(|v| v.to_string()).unwrap_or(String::new());
                if src_engage.contains("リンエンゲージ技") { enemy.set_engage_attack(Some("SID_リンエンゲージ技_威力減".into())); }
                else if src_engage.contains("ベレトエンゲージ技") { enemy.set_engage_attack(Some("SID_ベレトエンゲージ技_闇".into())); }
                else { enemy.set_engage_attack(replacement.get_engage_attack()); }
            }
        }
    }
    pub fn reset_sync_skill(&self) {
        self.emblem_data.get_god().get_level_data().unwrap().iter_mut().zip(self.emblem_data.level_data.iter())
            .for_each(|(level_data, data)| {
                data.sync_skills.set_skill_array(level_data.synchro_skills);
                data.engaged_skills.set_skill_array(level_data.engaged_skills);
        });
    }
    pub fn reset_engage_skill(&self) {
        self.emblem_data.get_god().get_level_data().unwrap().iter_mut().zip(self.emblem_data.level_data.iter())
            .for_each(|(level_data, data)| { data.engage_skills.set_skill_array(level_data.engaged_skills); });
    }
}
