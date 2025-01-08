use super::*;
use lazy_static::lazy_static;
use engage::gamedata::god::*;

lazy_static! {
pub static ref ENEMY_EMBLEMS: Vec<(&'static str, i32)> = vec![ 
    ("GID_M011_敵マルス", 0), ("GID_M017_敵マルス", 0),  ("GID_M021_敵マルス", 0), ("GID_M024_敵マルス", 0),
    ("GID_M002_シグルド", 1), ("GID_M011_敵シグルド", 1), ("GID_M017_敵シグルド", 1),
    ("GID_M011_敵セリカ", 2), ("GID_M017_敵セリカ", 2), ("GID_M020_敵セリカ", 2),
    ("GID_M011_敵ミカヤ", 3), ("GID_M017_敵ミカヤ", 3), ("GID_M019_敵ミカヤ", 3),
    ("GID_M011_敵ロイ", 4),   ("GID_M017_敵ロイ", 4), ("GID_M019_敵ロイ", 4),
    ("GID_M008_敵リーフ", 5), ("GID_M011_敵リーフ", 5), ("GID_M017_敵リーフ", 5),
    ("GID_M007_敵ルキナ", 6),
    ("GID_M010_敵リン", 7),
    ("GID_M010_敵ベレト", 9), ("GID_M014_敵ベレト", 9), 
    ("GID_E006_敵エーデルガルト", 12), ("GID_E006_敵ディミトリ", 12), ("GID_E006_敵クロード", 12),
   // ("GID_E001_敵チキ", 13), ("GID_E006_敵チキ", 13),
    ("GID_E002_敵ヘクトル", 14), ("GID_E005_敵ヘクトル", 14), ("GID_E006_敵ヘクトル", 14),
    ("GID_E003_敵ヴェロニカ", 15,), ("GID_E005_敵ヴェロニカ", 15), ("GID_E006_敵ヴェロニカ", 15),
    ("GID_E004_敵セネリオ", 16), ("GID_E006_敵セネリオ", 16), 
    ("GID_E004_敵カミラ", 17), ("GID_E006_敵カミラ", 17),
    ("GID_E005_敵クロム", 18), ("GID_E006_敵クロム", 18),
];
}

pub fn randomize_enemy_emblems() {
    let different_order = GameVariableManager::get_number("G_Emblem_Mode") != 0;
    if GameUserData::get_chapter().cid.contains("G00") && crate::utils::in_map_chapter() { return; }
    if unsafe { !super::super::STATUS.emblem_data_randomized } { return; }
    if unsafe { !super::super::STATUS.enemy_emblem_randomized } {
        ENEMY_EMBLEMS.iter().for_each(|&x|
            change_enemy_emblem_data(GodData::get_mut(x.0).unwrap(), x.1, different_order)
        );
        unsafe { super::super::STATUS.enemy_emblem_randomized = true };
    }
}

fn get_enemy_emblem_level(gids: &Il2CppString) -> (i32, i32, i32) {
    let gid = gids.to_string();
    if gid.contains( "M002") { return (1, 1, 1); }
    if gid.contains( "M007") { return (2, 3, 4); }
    if gid.contains( "M008") { return (3, 4, 5); }
    if gid.contains( "M010") { return (4, 5, 5); }
    if gid.contains( "M011") { return (1, 2, 3); }
    if gid.contains( "M014") || gid.contains( "M017") { return (5, 12, 14); }
    if gid.contains( "M019") || gid.contains( "M020") { return (10, 15, 17); }
    if gid.contains( "M021") { return (15, 17, 20); }
    if gid.contains( "M024") { return (20, 20, 20); }
    if gid.contains( "E005") { return (5, 10, 13); }
    if gid.contains( "E006") { return (10, 13, 15); }
    return (5, 10, 15);
}

fn change_enemy_emblem_data(enemy_god: &mut GodData, index: i32, different_order: bool) {
    if !different_order { return; }
    if let Some(source_god) = super::get_god_from_index(index, different_order){
        if source_god.gid.contains("チキ") { return; }
        if enemy_god.engage_attack.is_some() {
            if source_god.get_engage_attack().to_string() == "SID_ベレトエンゲージ技"  {
                enemy_god.set_engage_attack("SID_ベレトエンゲージ技_闇".into());
            }
            else if source_god.get_engage_attack().to_string() == "SID_リンエンゲージ技" { enemy_god.set_engage_attack("SID_リンエンゲージ技_威力減".into()); }
            else { enemy_god.set_engage_attack( source_god.get_engage_attack() );  }
        }
        enemy_god.link_gid = source_god.link_gid;
        enemy_god.engage_attack_link = source_god.engage_attack_link;
        enemy_god.ascii_name = source_god.ascii_name;
        enemy_god.mid = source_god.mid;
        enemy_god.nickname = source_god.nickname;
        enemy_god.sound_id = source_god.sound_id;
        
        if let Some(emblem_index) = unsafe { super::EMBLEM_LIST.iter().position(|&hash| hash == source_god.parent.hash) } {
            if enemy_god.gid.to_string() == "GID_M002_シグルド" || ( emblem_index == 8 || emblem_index == 10 || emblem_index == 11 ) || emblem_index > 18 {
                enemy_god.asset_id = source_god.asset_id;
            }
            else if emblem_index < 19 { enemy_god.asset_id = format!("敵{}", EMBLEM_ASSET[emblem_index as usize]).into(); }
        }
        enemy_god.face_icon_name = source_god.face_icon_name;
        enemy_god.face_icon_name_darkness = source_god.face_icon_name_darkness;
        enemy_god.ascii_name = source_god.ascii_name;
        enemy_god.unit_icon_id = source_god.unit_icon_id;
        let source_ggd = source_god.get_grow_table().unwrap();
        let src_data = GodGrowthData::get_level_data(&source_ggd.to_string()).unwrap();
        let max_level = src_data.len() as i32 - 1;
        let mut engage_skill = src_data[0].engage_skills[0].get_skill().unwrap();
        let has_engage_skill = if let Some(engage) = get_enemy_version_of_skills(engage_skill) {  
            engage_skill = engage; 
            true
        }
        else { false  };
        let emblem_level = get_enemy_emblem_level(enemy_god.gid);
        let mut levels: Vec<usize> = Vec::new();
        levels.push(emblem_level.2 as usize);
        levels.push(emblem_level.0 as usize);
        levels.push(emblem_level.1 as usize);
        levels.push(emblem_level.2 as usize);
        if let Some(ggd) = enemy_god.get_grow_table() {
            if let Some(level_data) = GodGrowthData::get_level_data(&ggd.to_string()) {
                level_data.iter_mut()
                    .for_each(|level|{
                        level.synchro_skills.clear();
                        level.engaged_skills.clear();
                        level.style_items.clear();
                        level.engage_skills.clear();
                        if has_engage_skill { level.engage_skills.add_skill(engage_skill, 5, 0); }
                    }
                );
                let mut level_iterator = levels.iter();
                level_data.iter_mut()
                    .for_each(|level|{
                        let index = level_iterator.next().cloned().unwrap_or(max_level as usize);
                        src_data[ index as usize ].synchro_skills.iter().for_each(|skill| if let Some(skill) = skill.get_skill() { level.synchro_skills.add_skill(skill, 5, 0); } );
                        src_data[ index as usize ].engaged_skills.iter().for_each(|skill| if let Some(skill) = skill.get_skill() { level.engaged_skills.add_skill(skill, 5, 0); } );
                        for z in 0..9 {
                            for w in 0..src_data[index as usize].style_items.items[z].len() {
                                let item = &src_data[index as usize].style_items.items[z].items[w];
                                level.style_items.add_item(z as i32, item);
                            }
                        }
                    }
                );
            }
        }
    }
}

// Nerfing some skills for pre-chapter 12
fn get_enemy_version_of_skills(skill: &'static SkillData) -> Option<&'static SkillData> {
    if !GameVariableManager::get_bool("G_Cleared_M020") {
        let sid = skill.sid.to_string();
        if sid.contains("SID_迅走") {  return SkillData::get("SID_迅走_闇")  }
        else if sid.contains("SID_増幅") {  return SkillData::get("SID_増幅_闇")  } 
        else if sid.contains("SID_超越") {  return SkillData::get("SID_超越_闇") }  // Rise Above -> Sink Below
        else if sid.contains("SID_踏ん張り") { return SkillData::get("SID_踏ん張り")  } // Hold Out -> Lowest Tier Hold Out
        else if sid.contains("SID_アイクエンゲージスキル") { return None; }   // Laguz Friend -> None
    }
    return Some(skill);
}
