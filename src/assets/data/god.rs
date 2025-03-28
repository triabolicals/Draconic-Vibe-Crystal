use super::*;

pub struct EngageAtkAsset {
    pub original_god_index: i32,
    pub engage_atk_hashes: Vec<i32>,
    pub entries: Vec<i32>,
}

pub struct GodAssets {
    pub gender: Gender,
    pub index: i32,
    pub mode: i32,
    pub entry_index: i32,
    pub darkness_index: i32,
}

impl EngageAtkAsset {
    pub fn new(_god: &GodData, god_index: i32, emblem_asset_index: i32, engage_atk_index: i32) -> Self {
        let sf = AssetTableStaticFields::get();
        Self {
            original_god_index: god_index,
            entries: sf.search_lists[2].iter()
                .filter(|entry| has_condition(entry, emblem_asset_index) && has_condition(entry, engage_atk_index))
                .map(|entry| entry.parent.index)
                .collect(),
            engage_atk_hashes: Vec::new(),
        }
    }
    pub fn is_engage_atk(&self, skill: &SkillData) -> bool {
        skill.style_skills.iter().any(|s| self.engage_atk_hashes.iter().any(|x| *x == s.parent.hash))
    }

    pub fn apply(&self, result: &mut AssetTableResult, unit: &Unit, gender_condition: i32){
        let mpid_condition = AssetTableStaticFields::get_condition_index(unit.person.get_name().unwrap());
        result.body_anims.clear();


        if let Some(entry) = self.entries.iter()
            .flat_map(|&index| AssetTable::try_index_get(index))
            .find(|entry| has_condition(entry, gender_condition) && has_condition(entry, mpid_condition))
        {
            result.commit_asset_table(entry);
            return;
        }
        let _ = self.entries.iter()
            .flat_map(|&index| AssetTable::try_index_get(index))
            .find(|entry| has_condition(entry, gender_condition))
            .map(|entry| result.commit_asset_table(entry));
    }
}


impl GodAssets {
    pub fn new(god_data: &GodData, mode: i32, index: i32) -> Self {
        let sf = AssetTableStaticFields::get();
        let gid_index =  AssetTableStaticFields::get_condition_index(god_data.gid);
        let darkness = AssetTableStaticFields::get_condition_index("闇化");
        let darkness_dlc = AssetTableStaticFields::get_condition_index(god_data.gid.to_string().replace("GID_", "GID_E006_敵"));
        let god_entry = sf.search_lists[mode as usize].iter().find(|entry| has_condition(entry, gid_index)).map_or_else(||-1, |f| f.parent.index);
        let dark_entry =
            sf.search_lists[mode as usize].iter().find(|entry| has_condition(entry, gid_index) && has_condition(entry, darkness))
            .map_or_else(
                ||sf.search_lists[mode as usize].iter().find(|entry| has_condition(entry, darkness_dlc) ).map_or_else(||-1, |f| f.parent.index),
                |f| f.parent.index
            );
        Self {
            gender: if god_data.female == 1 { Gender::Female } else { Gender::Male },
            index: index,
            mode: mode,
            entry_index: god_entry,
            darkness_index: dark_entry,
        }
    }
    pub fn get_entry(&self, is_darkness: bool) -> Option<&'static AssetTable> { 
        if is_darkness && self.darkness_index > 0 { AssetTable::try_index_get(self.darkness_index) }
        else { AssetTable::try_index_get(self.entry_index)  }
    }
}