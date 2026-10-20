use engage::app::{cookdata::*, reliancedata::*, structdictionary_1::*, relianceexpdata::*, HubMyRoomData};
use super::*;

pub struct PersonPool {
    pub playable: Vec<PlayableCharacter>,
    pub enemy_character: Vec<EnemyCharacter>,
    pub hash_to_index: HashMap<i32, i32>,
    pub pids: Vec<(String, String)>,
    init_cook_len: i32,
}
#[allow(unused)]
impl PersonPool {
    pub const PIDS: [&str; 41] = [
        "リュール", "ヴァンドレ", "クラン", "フラン", "アルフレッド", "エーティエ", "ブシュロン", "セリーヌ",
        "クロエ", "ルイ", "ユナカ", "スタルーク", "シトリニカ", "ラピス", "ディアマンド", "アンバー",
        "ジェーデ", "アイビー", "カゲツ", "ゼルコバ", "フォガート", "パンドロ", "ボネ", "ミスティラ",
        "パネトネ", "メリン", "オルテンシア", "セアダス", "ロサード", "ゴルドマリー", "リンデン",
        "ザフィーア", "ヴェイル", "モーヴ", "アンナ", "ジャン", "エル", "ラファール", "セレスティア",
        "グレゴリー", "マデリーン"
    ];
    pub const MPIDS: [&str; 64] = [
        "Lueur", "Vandre", "Clan", "Fram", "Alfred", "Etie", "Boucheron", "Celine",
        "Chloe", "Louis", "Yunaka", "Staluke", "Citrinica", "Lapis", "Diamand", "Umber",
        "Jade", "Ivy", "Kagetsu", "Zelkova", "Fogato", "Pandoro", "Bonet", "Misutira",
        "Panetone", "Merin", "Hortensia", "Seadas", "Rosado", "Goldmary", "Linden", "Saphir",
        "Veyre", "Mauve", "Anna", "Jean", "El", "Rafale", "Selestia", "Gregory",
        "Madeline", "Il", "Sepia", "Gris", "Marron", "Lumiere", "M003_Boss", "M004_Boss",
        "M005_Boss", "M006_Boss", "M013_BossA", "M013_BossB", "Sombre",
        "JeanFather", "S002_Boss","Eve","Morion","Hyacinth", "Sfoglia","AccessoriesShop",
        "BlackSmith","ItemShop","WeaponShop", "JeanMother"
    ];
    pub const VEYRES: [&str; 8] = [
        "ヴェイル_フード_顔出し", // Hooded but with Face
        "ヴェイル_黒_善_角折れ", // Broken Helm
        "ヴェイル_フード", // Hooded no face
        "ヴェイル_包帯",  // Normal
        "ヴェイル_白_悪", // white evil
        "ヴェイル_黒_悪", // black evil
        "ヴェイル_黒_善", // black good
        "M021_ヴェイル",    // Chapter 21 black evil
    ];
    pub const LUEUR: i32 = 276380359;
    pub const VANDRE: i32 = 152765422;
    pub const CLANNE: i32 = 1875144918;
    pub const FRAMME: i32 = 1654010808;
    pub const YUNAKA: i32 = 1172357650;
    pub const SEADALL: i32 = -266109647;
    pub const VEYLE: i32 = 356559395;
    pub const NEL: i32 = -1888964692;
    pub const RAFALE: i32 = 1696364213;
    pub const M002_LUMERA: i32 = 258677212;
    pub const ILLUSORY_DOUBLE: i32 = 1879825845;
    pub const M022_EMBLEMS: [i32; 11] = [
        -342947079, -1034406162, -103820870, 1375879663, -828765385,
        -1684363270, 1459767769, 377323117, -1059834855, -396338173, -1146313975
    ];
    pub const DIVINE_LUEUR: [i32; 3] = [ -936109577, 660489744, -484960500];
    pub fn get_grow_offset(person: PersonData) -> [i8; 9] {
        Self::get().playable.iter().find(|v| v.hash == person.hash())
            .map(|v| v.offset_o.clone())
            .unwrap_or([0i8; 9])
    }
    pub fn init() -> Self {
        let mut playable: Vec<_> = Self::PIDS.iter().map(|&p| PlayableCharacter::new(PersonData::get(p.into()))).collect();
        let hashes: HashMap<i32, i32> = playable.iter().enumerate().map(|(i, v)| (v.hash, i as i32)).collect();

        let mut count = 41;
        let cook_list = CookData::get_list();
        let init_cook_len = cook_list.iter().count() as i32;
        RelianceData::get_list().iter().flat_map(|p| to_opt(PersonData::get(p.get_pid())))
            .for_each(|p|{
                if !hashes.contains_key(&p.hash()) && to_opt(p.get_job()).is_some_and(|v| v.get_base().get(10) > 3) && count < 100 {
                    playable.push(PlayableCharacter::new(p));
                    println!("Playable Character Found: {} [#{}]", engage::app::Mess::get_game_data_name(p.get_pid()), count + 1);
                    count += 1;
                }
        });
        let mut hash_to_index: HashMap<i32, i32> = playable.iter().enumerate().map(|(i, p)|(p.hash, i as i32)).collect();
        let mut cursor = Cursor::new(include_bytes!("../data/person.bin"));
        let mut buffer: [u8; 5] = [0; 5];
        let mut count = 0;
        let mut enemy_character = vec![];
        while cursor.read_exact(&mut buffer).is_ok() {
            let idx = u8::from_be_bytes(buffer[0..1].try_into().unwrap());
            let hash = i32::from_be_bytes(buffer[1..].try_into().unwrap());
            if !hash_to_index.contains_key(&hash) {
                if let Some(person) = to_opt(PersonData::try_get_from_hash(hash)){
                    hash_to_index.insert(hash, idx as i32);
                    if !hashes.contains_key(&hash) {
                        if idx >= 41 && idx < 45 { enemy_character.push(EnemyCharacter::new(person, idx as i32 - 4)); }
                        else if (person.get_asset_force().value != 0 || idx > 35) && idx < 41 { enemy_character.push(EnemyCharacter::new(person, idx as i32)); }
                        hash_to_index.insert(hash, 100 + idx as i32);
                    }
                }
            }
            count += 1;
        }
        let pids = playable.iter().map(|v|{
            let data = v.get_person_data();
            (data.get_pid().to_rust_string(), data.get_ascii_name().to_rust_string())
        }).collect();
        PersonPool { playable, enemy_character, hash_to_index, pids, init_cook_len }
    }
    /*
    pub fn get_replacement_person_index(recruitment: i32, reverse: bool) -> Option<(i32, PersonData)> {
        Self::get_replacement_index(recruitment, reverse).zip(Self::get_replacement(recruitment, reverse, false))
    }
     */
    pub fn get_replacement_index(index: i32, reverse: bool) -> Option<i32> {
        let pool = Self::get();
        pool.playable.get(index as usize)
            .and_then(|s| s.get_replacement(reverse))
            .map(|v| v.hash())
            .and_then(|v| pool.playable.iter().position(|p| p.hash == v))
            .map(|i| i as i32)
    }
    pub fn get_replacement_index_2(pid: impl Into<Il2CppString>, reverse: bool) -> Option<i32> {
        to_opt(PersonData::get(pid.into()))
            .and_then(|p| Self::get_replacement_2(p, reverse))
            .map(|p| p.hash())
            .and_then(|v| Self::get().playable.iter().position(|p| p.hash == v))
            .map(|i| i as i32)
    }
    pub fn get_replacement_index_3(p: PersonData, reverse: bool) -> Option<i32> { 
        Self::get_replacement_2(p, reverse).map(|p| p.hash())
            .and_then(|v| Self::get().playable.iter().position(|p| p.hash == v))
            .map(|i| i as i32)
    }
    pub fn get_replacement(index: i32, reverse: bool, must_replace: bool) -> Option<PersonData> {
        if must_replace {
            if !Self::get_replacement_index(index, reverse).is_some_and(|i| i != index) { return None; }
        }
        Self::get().playable.get(index as usize).and_then(|v| v.get_replacement(reverse))
    }
    pub fn get_replacement_2(person: PersonData, reverse: bool) -> Option<PersonData> {
        let hash = person.hash();
        Self::get().playable.iter().find(|v| hash == v.hash).and_then(|v| v.get_replacement(reverse))
    }
    pub fn get_appearance_index(person: PersonData) -> Option<i32> {
        let p = Self::get();
        let phash = person.hash();
        let hash = p.enemy_character.iter().find(|v| v.hash == phash)
            .and_then(|p| p.get_enemy_replacement())
            .map(|p| p.hash()).unwrap_or(phash);
        
        Self::get().hash_to_index.get(&hash).cloned()
    }
    pub fn get_person_index(hash: i32) -> Option<i32> { Self::get().hash_to_index.get(&hash).cloned() }

    pub fn get() -> &'static Self { &GameData::get().person_pool }
    pub fn is_playable(person: PersonData) -> bool {
        let hash = person.hash();
        Self::get().playable.iter().any(|p| p.hash == hash)
    }
    pub fn is_enemy(person: PersonData) -> bool {
        let hash = person.hash();
        Self::get().enemy_character.iter().any(|p| p.hash == hash)
    }
    pub fn is_nil(person: PersonData) -> bool { Self::get_person_index(person.hash()).is_some_and(|v| (v % 100) == 41) }
    pub fn get_npc_veyle_replacement(person: PersonData, for_appearance: bool) -> Option<PersonData> {
        if for_appearance {
            let pid = person.get_pid().to_rust_string().trim_start_matches("PID_").to_string();
            if Self::VEYRES.contains(&pid.as_str()) {
                if Self::get_replacement_index(32, false).is_some_and(|v| v == 0) {
                    let lueur_pid = if DVCVariables::is_lueur_female() { "PID_青リュール_女性" } else { "PID_青リュール_男性" };
                    Some(PersonData::get(lueur_pid.into()))
                }
                else { PersonPool::get_replacement(32, false, true) }
            }
            else { None }
        }
        else { PersonPool::get_replacement(32, false, false) }
    }
    pub fn get_enemy_player_replacement(person: PersonData, appearance: bool) -> Option<PersonData> {
        let hash = person.hash();
        let index = Self::get_person_index(hash)?;
        match index {
            132 => { PersonPool::get_npc_veyle_replacement(person, appearance) }
            141..145 => { PersonPool::get_replacement(index -104, false, true) }
            _ => {
                Self::get().enemy_character.iter()
                    .find(|v| v.hash == hash)
                    .and_then(|v| v.get_enemy_replacement())
            }
        }
    }
    pub fn get_recruitment_index(&self, person_data: PersonData) -> Option<usize> { self.playable.iter().position(|v| v.hash == person_data.hash()) }
    pub fn get_playable_person_name(idx: i32) -> Il2CppString {
        let p = Self::get();
        if idx < p.playable.len() as i32 { Mess::get(p.playable[idx as usize].get_person_data().get_name()) }
        else { format!("Person #{}", idx).into() }
    }
    pub fn get_replacement_unit(idx: i32, reverse: bool) -> Option<Unit> {
        Self::get_replacement(idx, reverse, false).and_then(|v| to_opt(UnitPool::get_from_person(v, false)))
    }
    pub fn get_replacement_unit_2(pid: impl Into<Il2CppString>, reverse: bool) -> Option<Unit> {
        to_opt(PersonData::get(pid.into()))
            .and_then(|p| Self::get_replacement_2(p, reverse))
            .and_then(|p| to_opt(UnitPool::get_from_person(p, false)))
    }
    pub fn get_replacement_unit_3(person: PersonData, reverse: bool) -> Option<Unit> {
        Self::get_replacement_2(person, reverse).and_then(|p| to_opt(UnitPool::get_from_person(p, false)))
    }
    pub fn create_missing_data(person: PersonData) {
        let cook_list = CookData::get_list();
        let item_list = ItemData::get_list();
        let room_list = HubMyRoomData::get_list();
        let bento_example = ItemData::get("IID_ヴァンドレのお弁当".into());
        let item_dictionary = StructTemplate_1::<ItemData>::s_dictionary();
        let cook_dictionary = StructTemplate_1::<CookData>::s_dictionary();
        let room_dictionary = StructTemplate_1::<HubMyRoomData>::s_dictionary();
        if HubMyRoomData::get(person.get_pid()).is_null() {
            let new = HubMyRoomData::new();
            let index = HubMyRoomData::get_count();
            new.set_key(person.get_pid());
            new.set_hash(person.hash());
            new.set_index(index);
            room_dictionary.add(person.get_pid(), index);
            for i in 0..8{ unity::field_set_value_at_offset::<i32>(new, (10+i)*4, -1); }
            room_list.add(new);
        }
        if CookData::get(person.get_pid()).is_null() {
            let new_cook = CookData::new();
            let index = CookData::get_count();
            new_cook.set_index(index);
            new_cook.set_hash(person.hash());
            new_cook.set_pid(person.get_pid());
            cook_dictionary.add(person.get_pid(), index);
            let iid = format!("IID_{}お弁当", person.get_prefixless_pid());
            new_cook.set_bento_iid(iid.as_str());
            if ItemData::get(iid.as_str().into()).is_null() {
                let new_item = ItemData::new();
                let index_2 = ItemData::get_count();
                let iid = Il2CppString::new(iid.as_str());
                let hash = iid.get_hash_code();
                new_item.set_key(iid);
                new_item.set_iid(iid);
                new_item.set_name(format!("MIID_{}Bento", person.get_ascii_name()));
                new_item.set_help(format!("MIID_H_{}Bento", person.get_ascii_name()));
                new_item.set_index(index_2);
                new_item.set_hash(hash);
                new_item.set_kind(bento_example.get_kind());
                new_item.set_use_type(bento_example.get_use_type());
                new_item.set_icon(bento_example.get_icon());
                new_item.set_power(bento_example.get_power());
                new_item.set_endurance(1);
                new_item.set_add_type(bento_example.get_add_type());
                new_item.set_add_target(bento_example.get_add_target());
                new_item.set_price(bento_example.get_price());
                new_item.set_shoot_effect(bento_example.get_shoot_effect());
                new_item.set_hit_effect(bento_example.get_hit_effect());
                new_item.set_flag(bento_example.get_flag());
                new_item.set_add_effect(bento_example.get_add_effect());
                new_item.set_add_sids(bento_example.get_add_sids());
                new_item.set_add_help(bento_example.get_add_help());
                IItemDataMethods::on_build(new_item);
                IItemDataMethods::on_completed(new_item);
                item_dictionary.add_2(iid, index_2, hash);
                item_list.add(new_item);
            }
            cook_list.add(new_cook);
        }
    }
    pub fn fill_in_missing_data(&self) {
        let reliance_exp = Il2CppString::new("REXID_DVC");
        if RelianceExpData::get(reliance_exp).is_null() {
            let reliance_exp_dictionary = StructTemplate_1::<RelianceExpData>::s_dictionary();
            let new_exp = RelianceExpData::new();
            let index = RelianceExpData::get_count();
            new_exp.set_key(reliance_exp);
            new_exp.set_rexid(reliance_exp);
            new_exp.set_index(index);
            new_exp.set_hash(reliance_exp.get_hash_code());
            new_exp.set_exp_c(6);
            new_exp.set_exp_b(21);
            new_exp.set_exp_a(46);
            reliance_exp_dictionary.add(reliance_exp, index);
            RelianceExpData::get_list().add(new_exp);
        }
        self.playable.iter().map(|p| p.get_person_data())
            .for_each(|p|{ Self::create_missing_data(p); });
    }
    pub fn setup_reliance_data() {
        let hash = GameVariableManager::get_number(DVCVariable::HeroPerson.get_key());
        if let Some(dvc_exp) = to_opt(RelianceExpData::get("REXID_DVC".into())) {
            let exp_index = dvc_exp.get_index() as u8;
            if let Some(hero) = to_opt(PersonData::try_get_from_hash(hash))
                .and_then(|v| to_opt(RelianceData::get(v.get_pid())))
            {
                let index = hero.get_index() as usize;
                let exp = hero.get_exp_types();
                for i in 0..exp.len(){
                    if exp.get(i) == 0 && i != index { exp.set(i, exp_index); }
                }
            }
            RelianceData::get_list().iter().enumerate().filter(|&(i, _)| i > 40)
                .for_each(|(_, d)| {
                    if !PersonData::get(d.get_pid()).is_null() {
                        let exp = d.get_exp_types();
                        for i in 0..exp.len(){ if exp.get(i) == 0 { exp.set(i, exp_index); } }
                    }
                });
        }
    }
    pub fn reset_reliance_data() {
        if let Some(dvc_exp) = to_opt(RelianceExpData::get("REXID_DVC".into())) {
            let exp_index = dvc_exp.get_index() as u8;
            RelianceData::get_list().iter().for_each(|d| {
                let exp = d.get_exp_types();
                for i in 0..exp.len() { if exp.get(i) == exp_index { exp.set(i, 0); } }
            });
        }
    }
    pub fn randomize_cook_data(&self, cook_data: CookData) {
        if cook_data.is_null() { return; }
        if cook_data.index() < self.init_cook_len { return; }
        else {
            let rng = Random_2::get_hub();
            cook_data.set_taste1(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_taste1());
            cook_data.set_taste2(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_taste2());
            cook_data.set_taste3(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_taste3());
            cook_data.set_very_good_food(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_very_good_food());
            cook_data.set_good_food(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_good_food());
            cook_data.set_have_cooked_food(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_have_cooked_food());
            cook_data.set_challenging_food(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_challenging_food());
            cook_data.set_like_food(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_like_food());
            cook_data.set_dislike_food(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_dislike_food());
            cook_data.set_se_event(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).get_se_event());
            cook_data.set_mask_color075(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).mask_color075());
            cook_data.set_mask_color100(CookData::try_get_2(rng.get_value_2(self.init_cook_len)).mask_color100());
        }
    }
}


pub struct PlayableCharacter {
    pub hash: i32,
    pub skill: i32,
    pub female: bool,
    pub offset_o: [i8; 9],
}

impl PlayableCharacter {
    pub fn new(person: PersonData) -> PlayableCharacter {
        let skill = person.get_common_skills().m_list()
            .iter()
            .find_map(|s| to_opt(SkillData::try_get_2(s.value as i32 & 0xFFF)).filter(|s| s.get_flag().value & 1 == 0))
            .map(|s| s.hash())
            .unwrap_or(SkillData::try_get_2(1).hash());
        let female = person.get_gender().value == 2;
        let job_base = person.get_job().get_diff_grow();
        let target_level = if person.get_job().is_high() { 18 + person.get_level() } else { person.get_level() - 1 } as i32;
        let offset = person.get_offset_n();
        let mut offset_o = [0; 9];
        for i in 0..9 {
            let v = (offset.get(i as i32) as i32 * 100 - (job_base.get(i as i32) as i32) * target_level) / 100;
            offset_o[i] = v as i8;
        }
        PlayableCharacter{ hash: person.hash(), skill, female, offset_o}
    }
    pub fn get_recruitment_key(&self, reverse: bool ) -> String { format!("G_{}_{}", if reverse { "R2" } else { "R" }, self.get_person_data().get_pid()) }
    pub fn update_personal_skill(&self, to_new: bool) {
        let person = PersonData::try_get_from_hash(self.hash);
        let hash = if to_new { GameVariableManager::get_number(format!("G_P_{}", person.get_pid()).as_str()) } else { self.skill };
        change_personal_skill_hash(person.get_common_skills(), hash);
    }
    pub fn get_person_data(&self) -> PersonData{ PersonData::try_get_from_hash(self.hash) }
    pub fn get_replacement(&self, reverse: bool) -> Option<PersonData> {
        let key = self.get_recruitment_key(reverse);
        if GameVariableManager::is_exist(key.as_str()) { to_opt(PersonData::get(GameVariableManager::get_string(key.as_str()))) } else { Some(self.get_person_data()) }
    }
}

pub struct EnemyCharacter {
    pub hash: i32,
    pub playable_slot: Option<usize>,
}
impl EnemyCharacter {
    pub fn new(person: PersonData, playable_slot: i32) -> Self {
        let slot = if playable_slot < 41 { Some(playable_slot as usize) } else { None };
        person.get_flag().set_or_clear(true, 512);
        Self { hash: person.hash(), playable_slot: slot} // , common, normal, hard, lunatic }
    }
    pub fn get_data(&self) -> PersonData { PersonData::try_get_from_hash(self.hash) }
    pub fn get_enemy_replacement(&self) -> Option<PersonData> {
        if DVCVariable::UnitRecruitment.get_value() == 0 { None }
        else { self.playable_slot.and_then(|v| to_opt(PersonData::get(GameVariableManager::get_string(format!("G_R_PID_{}", PersonPool::PIDS[v]))))) }
    }
}
pub fn change_personal_skill_hash(skill: SkillArray, new_skill_hash: i32) {
    if let Some(new_skill) = to_opt(SkillData::try_get_from_hash(new_skill_hash)){ set_first_non_hidden_index(skill, new_skill.index()); }
}