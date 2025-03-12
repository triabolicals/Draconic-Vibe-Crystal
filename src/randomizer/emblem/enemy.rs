use super::*;
use std::sync::OnceLock;
pub static ENEMY_EMBLEMS: OnceLock<Vec<(i32, i32)>> = OnceLock::new(); 

pub fn initalize_dark_emblems() {
    ENEMY_EMBLEMS.get_or_init(||
        vec![ 
            (GodData::get_index("GID_M011_敵マルス"),0), (GodData::get_index("GID_M017_敵マルス"),0),  (GodData::get_index("GID_M021_敵マルス"),0), (GodData::get_index("GID_M024_敵マルス"),0),
            (GodData::get_index("GID_M002_シグルド"),1), (GodData::get_index("GID_M011_敵シグルド"),1), (GodData::get_index("GID_M017_敵シグルド"),1),
            (GodData::get_index("GID_M011_敵セリカ"),2), (GodData::get_index("GID_M017_敵セリカ"),2), (GodData::get_index("GID_M020_敵セリカ"),2),
            (GodData::get_index("GID_M011_敵ミカヤ"),3), (GodData::get_index("GID_M017_敵ミカヤ"),3), (GodData::get_index("GID_M019_敵ミカヤ"),3),
            (GodData::get_index("GID_M011_敵ロイ"),4),   (GodData::get_index("GID_M017_敵ロイ"),4), (GodData::get_index("GID_M019_敵ロイ"),4),
            (GodData::get_index("GID_M008_敵リーフ"),5), (GodData::get_index("GID_M011_敵リーフ"),5), (GodData::get_index("GID_M017_敵リーフ"),5),
            (GodData::get_index("GID_M007_敵ルキナ"),6),
            (GodData::get_index("GID_M010_敵リン"),7),
            (GodData::get_index("GID_M010_敵ベレト"),9), (GodData::get_index("GID_M014_敵ベレト"),9), 
            // (GodData::get_index("GID_E006_敵エーデルガルト"),12), (GodData::get_index("GID_E006_敵ディミトリ"),12), (GodData::get_index("GID_E006_敵クロード"),12),
               // (GodData::get_index("GID_E001_敵チキ"),13), (GodData::get_index("GID_E006_敵チキ"),13), 
            (GodData::get_index("GID_E002_敵ヘクトル"),14), (GodData::get_index("GID_E005_敵ヘクトル"),14), (GodData::get_index("GID_E006_敵ヘクトル"),14),
            (GodData::get_index("GID_E003_敵ヴェロニカ"),15,), (GodData::get_index("GID_E005_敵ヴェロニカ"),15), (GodData::get_index("GID_E006_敵ヴェロニカ"),15),
            (GodData::get_index("GID_E004_敵セネリオ"),16), (GodData::get_index("GID_E006_敵セネリオ"),16), 
            (GodData::get_index("GID_E004_敵カミラ"),17), (GodData::get_index("GID_E006_敵カミラ"),17),
            (GodData::get_index("GID_E005_敵クロム"),18), (GodData::get_index("GID_E006_敵クロム"),18),
        ]
    );
}    

pub fn randomize_enemy_emblems() {
    let different_order = GameVariableManager::get_number(DVCVariables::EMBLEM_RECRUITMENT_KEY) != 0;
    if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().emblem_data_randomized { return; }
    if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().enemy_emblem_randomized {
        println!("Randomizing Enemy Emblems");
        ENEMY_EMBLEMS.get().unwrap().iter().for_each(|&x|
            change_enemy_emblem_data(GodData::try_index_get_mut(x.0).unwrap(), x.1, different_order)
        );
        change_enemy_emblem_data(GodData::get_mut("GID_E006_敵エーデルガルト").unwrap(),12, false);
        change_enemy_emblem_data(GodData::get_mut("GID_E001_敵チキ").unwrap(), 13, false);
        change_enemy_emblem_data(GodData::get_mut("GID_E006_敵チキ").unwrap(), 13, false);
        let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.enemy_emblem_randomized = true );
    }
    adjust_enemy_edelgard_chapter();
}

fn get_enemy_emblem_level(gids: &Il2CppString) -> (i32, i32, i32) {
    let gid = gids.to_string();
    if gid.contains( "M002") || gid.contains("E001") { return (1, 2, 3); }
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

pub fn adjust_enemy_edelgard_chapter() {
    if !crate::randomizer::RANDOMIZER_STATUS.read().unwrap().enemy_emblem_randomized || crate::randomizer::RANDOMIZER_STATUS.read().unwrap().enemy_edelgard  { return; }
    let cid = GameUserData::get_chapter().get_prefixless_cid().to_string();
    if let Some(enemy_edelgard) = ENEMY_EMBLEMS.get().unwrap().iter().find(|w| 
        GodData::try_index_get(w.0).is_some_and(|g| g.gid.to_string().contains(cid.as_str())) && 
        DVCVariables::get_god_from_index(w.1, true).is_some_and(|god| god.parent.index == 64)) {
        let enemy_god = GodData::try_index_get_mut(enemy_edelgard.0).unwrap();
        let gid = if cid == "M002" { "GID_相手エーデルガルト" } else { "GID_E006_敵エーデルガルト" };
        let other_edelgard = &mut GodData::get_mut(gid).unwrap();
        // let other_edelgard = god.unwrap();
        enemy_god.change_data = other_edelgard.change_data;
        other_edelgard.change_data.iter().for_each(|god|{
            god.set_main_data(enemy_god);
            god.set_engage_attack(enemy_god.get_engage_attack());
        });
    }
    let _ = crate::randomizer::RANDOMIZER_STATUS.try_write().map(|mut lock| lock.enemy_edelgard  = true );
}

pub fn adjust_enemy_edelgard_post_chapter() {
    let edelgard = GodData::get_mut("GID_エーデルガルト").unwrap();
    let enemy_edelgard = GodData::get_mut("GID_相手エーデルガルト").unwrap();
    enemy_edelgard.change_data.iter().for_each(|god|{
        god.set_main_data(enemy_edelgard);
        god.set_engage_attack(edelgard.get_engage_attack());
    });
    let fx_edelgard = GodData::get_mut("GID_E006_敵エーデルガルト").unwrap();
    fx_edelgard.change_data.iter().for_each(|god|{
        god.set_main_data(fx_edelgard);
        god.set_engage_attack(edelgard.get_engage_attack());
    });
}
fn change_enemy_emblem_data(enemy_god: &mut GodData, index: i32, different_order: bool) {
    // if !different_order { return; }
    if let Some(source_god) = DVCVariables::get_god_from_index(index, different_order){
        if enemy_god.engage_attack.is_some() {
            if source_god.get_engage_attack().to_string() == "SID_ベレトエンゲージ技"  {
                enemy_god.set_engage_attack("SID_ベレトエンゲージ技_闇".into());
            }
            else if source_god.get_engage_attack().to_string() == "SID_リンエンゲージ技" { enemy_god.set_engage_attack("SID_リンエンゲージ技_威力減".into()); }
            else { enemy_god.set_engage_attack( source_god.get_engage_attack() );  }
        }
    //if different_order {
        enemy_god.link_gid = source_god.link_gid;
        enemy_god.engage_attack_link = source_god.engage_attack_link;
        enemy_god.ascii_name = source_god.ascii_name;
        enemy_god.mid = source_god.mid;
        enemy_god.nickname = source_god.nickname;
        enemy_god.sound_id = source_god.sound_id;
        let m002 = enemy_god.gid.to_string().contains("M002");
        if let Some(emblem_index) = super::EMBLEM_LIST.get().unwrap().iter().position(|&hash| hash == source_god.parent.hash) {
            if m002 || ( emblem_index == 8 || emblem_index == 10 || emblem_index == 11 ) || emblem_index > 18 {
                enemy_god.asset_id = source_god.asset_id;
            }
            else if emblem_index < 19 { enemy_god.asset_id = format!("敵{}", EMBLEM_ASSET[emblem_index as usize]).into(); }
        }

        enemy_god.face_icon_name = source_god.face_icon_name;
        enemy_god.face_icon_name_darkness = source_god.face_icon_name_darkness;
        enemy_god.ascii_name = source_god.ascii_name;
        enemy_god.unit_icon_id = source_god.unit_icon_id;
        //}

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
        levels.push(emblem_level.0 as usize);
        levels.push(emblem_level.1 as usize);
        levels.push(emblem_level.2 as usize);
        if let Some(ggd) = enemy_god.get_grow_table() {
            if let Some(level_data) = GodGrowthData::get_level_data(&ggd.to_string()) {
                level_data.iter_mut()
                    .for_each(|level|{
                        level.synchro_skills.clear();
                        level.engaged_skills.clear();
                        if index != 13 { level.style_items.clear(); }
                        level.engage_skills.clear();
                        if has_engage_skill { level.engage_skills.add_skill(engage_skill, 5, 0); }
                    }
                );
                let mut level_iterator = levels.iter();
                level_data.iter_mut()
                    .for_each(|level|{
                        let index_c = level_iterator.next().cloned().unwrap_or(max_level as usize);
                        src_data[ index as usize ].synchro_skills.iter().for_each(|skill|
                            if let Some(skill) = skill.get_skill() { 
                                { level.synchro_skills.add_skill(skill, 5, 0);  }
                            }
                        );
                        src_data[ index as usize ].engaged_skills.iter().for_each(|skill| 
                            if let Some(skill) = skill.get_skill() { 
                                //if skill.parent.hash != 924387794 
                                { level.engaged_skills.add_skill(skill, 5, 0);  }
                            }
                        );
                        if index != 13 {
                            for z in 0..9 {
                                let src_items = src_data[ index_c as usize ].style_items.get_items(z); 
                                src_items.iter().for_each(|item|{ level.style_items.add_item(z, item); });
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
    if !DVCVariables::is_main_chapter_complete(20) {
        let sid = skill.sid.to_string();
        if sid.contains("SID_迅走") {  return SkillData::get("SID_迅走_闇")  }
        else if sid.contains("SID_増幅") {  return SkillData::get("SID_増幅_闇")  } 
        else if sid.contains("SID_超越") {  return SkillData::get("SID_超越_闇") }  // Rise Above -> Sink Below
        else if sid.contains("SID_踏ん張り") { return SkillData::get("SID_踏ん張り")  } // Hold Out -> Lowest Tier Hold Out
        else if sid.contains("SID_アイクエンゲージスキル") { return None; }   // Laguz Friend -> None
    }
    return Some(skill);
}
#[unity::from_offset("App", "GodData", "CalcChangeData")]
fn calc_change_data(this: &GodData, method_info: OptionalMethod) -> Option<&Array<&GodData>>;