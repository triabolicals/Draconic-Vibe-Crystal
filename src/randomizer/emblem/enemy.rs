use super::*;
use lazy_static::lazy_static;
use engage::gamedata::god::*;
use engage::mess::*;

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
    ("GID_E001_敵チキ", 13), ("GID_E006_敵チキ", 13),
    ("GID_E002_敵ヘクトル", 14), ("GID_E005_敵ヘクトル", 14), ("GID_E006_敵ヘクトル", 14),
    ("GID_E003_敵ヴェロニカ", 15,), ("GID_E005_敵ヴェロニカ", 15), ("GID_E006_敵ヴェロニカ", 15),
    ("GID_E004_敵セネリオ", 16), ("GID_E006_敵セネリオ", 16), 
    ("GID_E004_敵カミラ", 17), ("GID_E006_敵カミラ", 17),
    ("GID_E005_敵クロム", 18), ("GID_E006_敵クロム", 18),
];
}

pub fn randomize_enemy_emblems() {
    // Engage Attacks
    let different_order = GameVariableManager::get_number("G_Emblem_Mode") != 0;
    ENEMY_EMBLEMS.iter().for_each(|&x|
        change_enemy_emblem_data(GodData::get_mut(x.0).unwrap(), x.1, different_order)
    );

}

fn get_enemy_emblem_level(gid: &Il2CppString) -> i32 {
    if str_contains(gid, "M002") { return 2; }
    if str_contains(gid, "M007") { return 3; }
    if str_contains(gid, "M008") { return 5; }
    if str_contains(gid, "M010") { return 6; }
    if str_contains(gid, "M011") { return 3; }
    if str_contains(gid, "M014") || str_contains(gid, "M017") || str_contains(gid, "M019") ||  str_contains(gid, "M020") { return 13; }
    if str_contains(gid, "M021") { return 15; }
    if str_contains(gid, "M024") { return 20; }
    if str_contains(gid, "E005") { return 13; }
    if str_contains(gid, "E006") { return 16; }
    return 10;
}

fn change_enemy_emblem_data(enemy_god: &mut GodData, index: i32, different_order: bool) {
    let emblem_index =  if different_order { EMBLEM_ORDER.lock().unwrap()[index as usize] } else { index };
    if emblem_index < 0 || emblem_index > 19 { return; }
    let source_god = GodData::get(EMBLEM_GIDS[emblem_index as usize]).unwrap();
    if enemy_god.engage_attack.is_some() {
        if source_god.get_engage_attack().to_string() == "SID_ベレトエンゲージ技"  {
            enemy_god.set_engage_attack("SID_ベレトエンゲージ技_闇".into());
        }
        else if source_god.get_engage_attack().to_string() == "SID_リンエンゲージ技" {
            enemy_god.set_engage_attack("SID_リンエンゲージ技_威力減".into());
        }
        else { enemy_god.set_engage_attack( source_god.get_engage_attack() );  }
    }

    enemy_god.link_gid = source_god.link_gid;
    enemy_god.engage_attack_link = source_god.engage_attack_link;
    enemy_god.ascii_name = source_god.ascii_name;
    enemy_god.mid = source_god.mid;
    enemy_god.nickname = source_god.nickname;
    enemy_god.sound_id = source_god.sound_id;
    if enemy_god.gid.to_string() == "GID_M002_シグルド" || ( emblem_index == 8 || emblem_index == 10 || emblem_index == 11 ) {
         enemy_god.asset_id = source_god.asset_id;
    }
    else {
        enemy_god.asset_id = format!("敵{}", EMBLEM_ASSET[emblem_index as usize]).into();
    }
    enemy_god.face_icon_name = source_god.face_icon_name;
    enemy_god.face_icon_name_darkness = source_god.face_icon_name_darkness;
    enemy_god.ascii_name = source_god.ascii_name;
    enemy_god.unit_icon_id = source_god.unit_icon_id;
    enemy_god.on_complete();
    let source_ggd = source_god.get_grow_table().unwrap();
    let src_data = GodGrowthData::get_level_data(&source_ggd.to_string()).unwrap();
    let mut engage_skill = src_data[0].engage_skills[0].get_skill().unwrap();
    if let Some(engage) = get_enemy_version_of_skills(engage_skill) {   engage_skill = engage; }
    let emblem_level = get_enemy_emblem_level(enemy_god.gid);
    if let Some(ggd) = enemy_god.get_grow_table() {
        if let Some(level_data) = GodGrowthData::get_level_data(&ggd.to_string()) {
            for y in 0..level_data.len() {
                level_data[y].synchro_skills.clear();
                level_data[y].engaged_skills.clear();
                level_data[y].style_items.clear();
                level_data[y].engage_skills.clear();
                for z in 0..src_data[emblem_level as usize].synchro_skills.list.size {
                    let skill = src_data[emblem_level as usize].synchro_skills[z as usize].get_skill().unwrap();
                    level_data[y as usize ].synchro_skills.add_skill(skill, 5, 0); 

                }
                for z in 0..src_data[emblem_level as usize].engaged_skills.list.size {
                    let skill = src_data[emblem_level as usize].engaged_skills[z as usize].get_skill().unwrap();
                    level_data[y as usize ].engaged_skills.add_skill(skill, 5, 0); 
                }
                level_data[y].engage_skills.add_skill(engage_skill, 5, 0);
                for z in 0..9 {
                    for w in 0..src_data[emblem_level as usize].style_items.items[z].len() {
                        let item = &src_data[emblem_level as usize].style_items.items[z].items[w];
                        level_data[y].style_items.add_item(z as i32, item);
                    }
                }
            }
        }
    }
}

// Nerfing some skills for pre-chapter 12
fn get_enemy_version_of_skills(skill: &SkillData) -> Option<&'static SkillData> {
    if !GameVariableManager::get_bool("G_Cleared_M020") {
        if skill.sid.contains("SID_迅走") {  return SkillData::get("SID_迅走_闇")  }
        if skill.sid.contains("SID_増幅") {  return SkillData::get("SID_増幅_闇")  } 
        if skill.sid.contains("SID_超越") {  return SkillData::get("SID_超越_闇") }  // Rise Above -> Sink Below
        if skill.sid.contains("SID_踏ん張り") { return SkillData::get("SID_踏ん張り")  } // Hold Out -> Lowest Tier Hold Out
        if skill.sid.contains("SID_アイクエンゲージスキル") { return SkillData::get("SID_無し") }   // Laguz Friend -> None
    }
    None
}
