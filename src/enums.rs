use engage::gamedata::{*, skill::*, item::ItemData};
use std::sync::Mutex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::io::Write;
use engage::mess::Mess;
pub static SKILL_BLACK_LIST: Mutex<Vec<i32>> = Mutex::new(Vec::new()); 
pub static ITEM_BLACK_LIST: Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static PERSONAL_BLIST: Mutex<Vec<i32>> = Mutex::new(Vec::new());
pub static SET_RECRUITMENT: Mutex<Vec<(i32, i32, bool)>> = Mutex::new(Vec::new());
pub static mut UNIT_RANDOM: bool = false;
pub static mut EMBLEM_RANDOM: bool = false;
pub static mut LUEUR_CHANGE: bool = false; 
pub const IS_GHAST: bool = false;

pub const MPIDS: [&str;41] = ["MPID_Lueur", "MPID_Vandre", "MPID_Clan", "MPID_Fram", "MPID_Alfred", "MPID_Etie", "MPID_Boucheron", "MPID_Celine", "MPID_Chloe", "MPID_Louis", "MPID_Yunaka", "MPID_Staluke", "MPID_Citrinica", "MPID_Lapis", "MPID_Diamand", "MPID_Umber", "MPID_Jade", "MPID_Ivy", "MPID_Kagetsu", "MPID_Zelkova", "MPID_Fogato", "MPID_Pandoro", "MPID_Bonet", "MPID_Misutira", "MPID_Panetone", "MPID_Merin", "MPID_Hortensia", "MPID_Seadas", "MPID_Rosado", "MPID_Goldmary", "MPID_Linden", "MPID_Saphir", "MPID_Veyre", "MPID_Mauve", "MPID_Anna", "MPID_Jean", "MPID_El", "MPID_Rafale", "MPID_Selestia", "MPID_Gregory", "MPID_Madeline"];
pub const PIDS: [&str; 41] = ["PID_リュール", "PID_ヴァンドレ", "PID_クラン", "PID_フラン", "PID_アルフレッド", "PID_エーティエ", "PID_ブシュロン", "PID_セリーヌ", "PID_クロエ", "PID_ルイ", "PID_ユナカ", "PID_スタルーク", "PID_シトリニカ", "PID_ラピス", "PID_ディアマンド", "PID_アンバー", "PID_ジェーデ", "PID_アイビー", "PID_カゲツ", "PID_ゼルコバ", "PID_フォガート", "PID_パンドロ", "PID_ボネ", "PID_ミスティラ", "PID_パネトネ", "PID_メリン", "PID_オルテンシア", "PID_セアダス", "PID_ロサード", "PID_ゴルドマリー", "PID_リンデン", "PID_ザフィーア", "PID_ヴェイル", "PID_モーヴ", "PID_アンナ", "PID_ジャン", "PID_エル", "PID_ラファール", "PID_セレスティア", "PID_グレゴリー", "PID_マデリーン"];
pub const RECRUIT_CID : [&str; 41] = ["M001", "M001", "M002", "M002", "M003", "M003", "M003", "M004", "M004", "M004", "M006", "M007", "M007", "M007", "M008", "M008", "M009", "M011", "M011", "M011", "M012", "M012", "M012", "M013", "M013", "M013", "M014", "M015", "M016", "M016", "M018", "M019", "M022", "M021", "S002", "S001", "E006", "E006", "E006", "E006", "E006"];
// Emblem and Emblem SKill Related
pub const RR_ORDER: [u8; 41] = [32, 33, 31, 30, 29, 28, 27, 26, 25, 24, 22, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 34, 10, 35, 9, 8, 7, 6, 5, 4, 3, 2, 0, 1, 21, 23, 36, 37, 38, 39, 40];
pub const EMBLEM_ASSET: [&str; 24] = ["マルス", "シグルド", "セリカ", "ミカヤ", "ロイ", "リーフ", "ルキナ", "リン", "アイク", "ベレト", "カムイ", "エイリーク", "エーデルガルト", "チキ", "ヘクトル", "ヴェロニカ", "セネリオ", "カミラ", "クロム", "リュール", "ディミトリ", "クロード", "ルフレ", "エフラム"];
pub const EIRIKA_TWIN_SKILLS: [&str; 12] = [ "SID_月の腕輪", "SID_太陽の腕輪", "SID_日月の腕輪", "SID_優風", "SID_勇空", "SID_蒼穹", "SID_月の腕輪＋", "SID_太陽の腕輪＋", "SID_日月の腕輪＋", "SID_優風＋", "SID_勇空＋", "SID_蒼穹＋" ];
pub const EMBLEM_GIDS: [&str; 19] = ["GID_マルス", "GID_シグルド", "GID_セリカ", "GID_ミカヤ", "GID_ロイ", "GID_リーフ", "GID_ルキナ", "GID_リン", "GID_アイク", "GID_ベレト", "GID_カムイ", "GID_エイリーク", "GID_エーデルガルト", "GID_チキ", "GID_ヘクトル", "GID_ヴェロニカ", "GID_セネリオ", "GID_カミラ", "GID_クロム"];
pub const RINGS: [&str; 23] = ["Marth", "Siglud", "Celica", "Micaiah", "Roy", "Leaf", "Lucina", "Lin", "Ike", "Byleth", "Kamui", "Eirik", "Edelgard", "Tiki", "Hector", "Veronica", "Senerio", "Camilla", "Chrom", "Dimitri", "Claude", "Reflet", "Ephraim"];
pub const EMBELM_PARA: [&str; 19] = ["S014", "S009", "S013", "S011", "S012", "S010", "S003", "S004", "S005", "S006", "S007", "S008", "G007", "G001", "G002", "G003", "G004", "G005", "G006"];
pub const UNLOCK_PARA: [&str; 19] = ["M022", "M017", "M020", "M019", "M019", "M017", "M011", "M012", "M013", "M014", "M015", "M016", "", "G001", "G002", "G003", "G004", "G005", "G006"];
pub const PARA_LEVEL: [i32; 19] = [35, 28, 32, 28, 31, 30, 21, 22, 25, 28, 26, 29, -1, -1, -1, -1, -1, -1, -1];
pub const ENGAGE_ATK_AI: [&str; 11] = ["AI_AT_EngageAttack","AI_AT_EngagePierce", "AI_AT_EngageVision", "AI_AT_EngageDance", "AI_AT_EngageOverlap", "AI_AT_EngageBless", "AI_AT_EngageWaitGaze", "AI_AT_EngageSummon", "AI_AT_EngageCamilla", "AI_AT_Versus", "AI_AT_EngageWait"];
// Items

// Emblem Cutscene Related
pub const KENGEN: [&str; 19] = ["Scene03", "Kengen01", "Kengen02", "Kengen03", "Kengen04", "Kengen05", "Kengen07", "Kengen06", "Kengen08", "Kengen09", "Kengen10", "Kengen11", "Kengen13", "Kengen14", "Kengen15", "Kengen19", "Kengen16", "Kengen17", "Kengen18"];
pub const STYLE_NAMES: [&str; 9] = ["連携スタイル", "騎馬スタイル", "隠密スタイル", "重装スタイル",  "飛行スタイル", "魔法スタイル", "気功スタイル", "竜族スタイル", "スタイル無し"];
pub const BLACKLIST_ITEMS: [&str; 25] = [
    "IID_マスタープルフ", "IID_リベラシオン改", "IID_リベラシオン改_ノーマル",
    "IID_リベラシオン", "IID_リベラシオン_M000", "IID_無し", "IID_不明", "IID_エンゲージ枠", "IID_火炎砲台", "IID_牙", "IID_邪竜石_E",
    "IID_邪竜石_E005", "IID_邪竜石_魔法攻撃_E", "IID_イル_反撃", "IID_イル_薙払いビーム", "IID_イル_突進",
    "IID_イル_吸収", "IID_イル_召喚", "IID_火のブレス", "IID_炎塊", "IID_ソンブル_物理攻撃",
    "IID_ソンブル_魔法攻撃", "IID_ソンブル_回転アタック", "IID_ソンブル_ビーム", "IID_ソンブル_エンゲージブレイク", 
];
pub const EXTRA_SYNCS: [&str; 24] = [ 
"SID_計略_引込の計", "SID_計略_猛火計", "SID_計略_聖盾の備え", "SID_計略_毒矢", "SID_回復", "SID_風薙ぎ", "SID_轟雷", "SID_業火", "SID_剛腕", "SID_祈り", 
"SID_ダイムサンダ", "SID_いやしの心", "SID_引き寄せ", "SID_ギガスカリバー", "SID_旋風", "SID_体当たり", "SID_閃花", "SID_幻月", "SID_風神", "SID_騎士道", 
"SID_武士道", "SID_絆の指輪_アルフォンス", "SID_絆の指輪_シャロン", "SID_絆の指輪_アンナ" ];

// Skill Related
pub const STYLE: [&str;9] = ["Default", "Cooperation", "Horse", "Covert", "Heavy", "Fly", "Magic","Prana", "Dragon"];

pub const BLACKLIST_SKILL: &[&str] = &[
    "SID_バリア１", "SID_バリア２", "SID_バリア３", "SID_バリア４",
    "SID_バリア１_ノーマル用", "SID_バリア２_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア４_ノーマル用",
    "SID_異界の力_閉", "SID_異界の力_炎", "SID_異界の力_死", "SID_異界の力_夢", "SID_異界の力_科", "SID_守護者_E001", 
    "SID_守護者_E002", "SID_守護者_E003", "SID_守護者_E004", "SID_計略", "SID_無し", "SID_切磋琢磨", "SID_オルタネイト", "SID_双聖", "SID_竜化_無効", "SID_虚無の呪い" ];

pub const MADDENING_BLACK_LIST: &[&str] = &[
    "SID_杖使い＋＋", "SID_杖使い＋", "SID_杖使い", "SID_残像", "SID_日月の腕輪", "SID_慈悲", "SID_計略_引込の計", "SID_計略_猛火計", "SID_計略_聖盾の備え", "SID_計略_毒矢", 
    "SID_守護者", "SID_守護者_使用不可", "SID_全弾発射", "SID_輸送隊", "SID_裏邪竜ノ子_兵種スキル", "SID_負けず嫌い", "SID_竜脈・異", "SID_先生", "SID_増幅_闇", "SID_重唱", "SID_大好物",
    "SID_熟練者", "SID_ブレイク無効", "SID_師の導き", "SID_拾得", "SID_竜脈", "SID_特別な踊り", "SID_契約", "SID_七色の叫び＋", "SID_七色の叫び", "SID_戦場の花", "SID_平和の花", "SID_大盤振る舞い", "SID_料理再現",
    "SID_一攫千金", "SID_努力の才", "SID_白の忠誠", "SID_碧の信仰", "SID_緋い声援", "SID_筋肉増強剤", "SID_虚無の呪い", "SID_自壊", "SID_自己回復", "SID_角の睨み", "SID_囮指名", "SID_戦技", "SID_血統", 
    "SID_引き寄せ", "SID_体当たり", "SID_入れ替え", "SID_異形狼連携", "SID_幻影狼連携", "SID_星玉の加護", "SID_双聖", 
];
pub const ENGAGE_PREFIX: [&str; 29] = [ "Mar", "Sig", "Cel", "Mic", "Roy", "Lei", "Luc", "Lyn", "Ike", "Byl", "Cor", "Eir", "Thr", "Tik", "Hec", "Ver", "Sor", "Cmi", "Chr", "Ler", "Ede", "Dim", "Cla", "Eph", "Enb", "Wng2D", "Cav2B", "Com0A", "Com0B" ];

pub const PERSONAL_BLACK_LIST: &[&str] = &[ "SID_瘴気の領域", "SID_異形狼連携", "SID_幻影狼連携", "SID_全弾発射",  "SID_守護者_E001", "SID_守護者_E002", "SID_守護者_E003", "SID_守護者_E004", "SID_守護者_使用不可"];

pub const NO_INHERITS: &[&str] = &[
    "SID_熟練者", "SID_熟練者＋", "SID_虚無の呪い", "SID_特効耐性", "SID_特効無効", "SID_不動", "SID_自壊", "SID_噛描", "SID_狂乱の一撃", "SID_バリア１", "SID_バリア２", "SID_バリア３", "SID_バリア４", "SID_バリア１_ノーマル用",    
    "SID_バリア２_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア４_ノーマル用", "SID_チェインアタック威力軽減", "SID_チェインアタック威力軽減＋", "SID_自壊"];

fn is_valid_skill(sid: &str ) -> bool {
    if let Some(skill) = SkillData::get(sid) {
        if SKILL_BLACK_LIST.lock().unwrap().iter().find(|x| **x ==  skill.parent.index).is_some() { return false; }
        if skill.help.is_none() { return false; }
        else if  Mess::get( skill.name.unwrap() ).to_string().len() == 0 { return false; }
        if skill.name.is_none() { return false; }
        else if Mess::get( skill.help.unwrap() ).to_string().len() == 0 { return false; }
        if skill.is_style_skill() { return false; }
        return  skill.get_flag() & 511 == 0;
    }
    return false;
}
fn is_valid_person(pid: &str) -> i32 {
    let mut out = 0;
    for x in PIDS {
        if pid == x { return out; } // by pid
        let person = PersonData::get(x).expect("Invalid person.");
        let mpid = person.get_name().unwrap();
        if crate::utils::str_contains(mpid, pid) { return out; }  // by mpid
        if crate::utils::str_contains(Mess::get(mpid), pid) { return out; } //by mpid value
        out += 1;
    }
    if pid == "Alear" { return 0; }
    return -1;
}
fn is_valid_emblem(gid: &str) -> i32 {
    let mut out = 0;
    for x in EMBLEM_GIDS {
        if gid == x { return out; } // by gid
        let god = GodData::get(x).unwrap();
        let mpid = god.mid;
        if crate::utils::str_contains(mpid, gid) { return out; }  // by mid
        if crate::utils::str_contains(Mess::get(mpid), gid) { return out; } // by mpid value
        out += 1;
    }
    return -1;

}
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn generate_black_list() {
    SKILL_BLACK_LIST.lock().unwrap().clear();
    ITEM_BLACK_LIST.lock().unwrap().clear();
    PERSONAL_BLIST.lock().unwrap().clear();
    SET_RECRUITMENT.lock().unwrap().clear();
    
    BLACKLIST_SKILL.iter().for_each(|x| if let Some(skill) = SkillData::get(x) { SKILL_BLACK_LIST.lock().unwrap().push(skill.parent.index); });
    BLACKLIST_ITEMS.iter().for_each(|x| if let Some(item) = ItemData::get(x) { ITEM_BLACK_LIST.lock().unwrap().push(item.parent.index);  });
    PERSONAL_BLACK_LIST.iter().for_each(|x| if let Some(skill) = SkillData::get(x) { PERSONAL_BLIST.lock().unwrap().push(skill.parent.index); });

    if let Ok(lines) = read_lines("sd:/engage/config/DVC_List.txt") {
        println!("Reading from sd:/engage/config/DVC_List.txt");
        for line in lines.flatten() {
            let spilt: Vec<_> = line.split_whitespace().collect();
            if spilt.len() > 1 {
                if spilt[0] == "*" || spilt[0] == "--" { continue; }    // comments
                match spilt[0] {
                    "remove_skill" => { 
                        for z in 1..spilt.len() {
                            if let Some(skill) = SkillData::get(&spilt[z]) {
                                let index = skill.parent.index;
                                if  SKILL_BLACK_LIST.lock().unwrap().iter().find(|&x| *x == index).is_none() {
                                    println!("Added General Skill Blacklist #{}: {}", index, crate::utils::get_skill_name(skill));
                                    SKILL_BLACK_LIST.lock().unwrap().push(index);
                                }
                            }
                        }
                    },
                    "remove_personal_skill" => { 
                        for z in 1..spilt.len() {
                            if let Some(skill) = SkillData::get(&spilt[z]) {
                                let index = skill.parent.index;
                                if PERSONAL_BLIST.lock().unwrap().iter().find(|&x| *x == index).is_none() && SKILL_BLACK_LIST.lock().unwrap().iter().find(|&x| *x == index).is_none() {
                                    println!("Added to Personal Skill Blacklist #{}: {}", index, crate::utils::get_skill_name(skill));
                                    PERSONAL_BLIST.lock().unwrap().push(index);
                                } 
                            }
                        }
                     },
                    "remove_item" => {                     
                        for z in 1..spilt.len() {
                            if let Some(item) = ItemData::get(&spilt[z]) {
                                let index = item.parent.index; 
                                if ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == index).is_none() {
                                    println!("Added General Item Blacklist #{}: {}", index, crate::utils::get_item_name(item));
                                    ITEM_BLACK_LIST.lock().unwrap().push(index);
                                }
                            }
                        }
                    },
                    "unit_swap" => {
                        if spilt.len() < 3 { 
                            if spilt[1] == "random" {  unsafe { UNIT_RANDOM = true; } }
                            continue;
                        }
                        if spilt[1] == "random" {  unsafe { UNIT_RANDOM = true; } continue; }
                        let person_index = is_valid_person(spilt[1]);
                        if person_index == -1 { continue; }
                        let new_person_index = is_valid_person(spilt[2]);
                        if new_person_index == -1 { continue; }
                        if person_index == 0 && new_person_index > 35 { continue; }
                        if ( person_index >= 36 || new_person_index >= 36 ) && !crate::utils::dlc_check() { continue; }
                        SET_RECRUITMENT.lock().unwrap().push( (person_index, new_person_index, false));
                        println!("Unit {} is swapped with Unit {}", person_index, new_person_index);
                    },
                    "emblem_swap" => {
                        if spilt.len() < 3 { 
                            if spilt[1] == "random" {  unsafe { EMBLEM_RANDOM = true; } }
                            continue;
                        }
                        if spilt[1] == "random" {  unsafe { EMBLEM_RANDOM = true; } continue; }
                        let emblem_index = is_valid_emblem(spilt[1]);
                        if emblem_index == -1 { continue; }
                        let new_emblem_index = is_valid_emblem(spilt[2]);
                        if new_emblem_index == -1 { continue; }
                        if ( emblem_index >= 12 || new_emblem_index >= 12 ) && !crate::utils::dlc_check() { continue; }
                        SET_RECRUITMENT.lock().unwrap().push( (emblem_index, new_emblem_index, true));
                        println!("Emblem {} is swapped with Emblem {}", emblem_index, new_emblem_index);
                    },
                    _ => { },
                }
            }
        }
    }
    else {
        println!("Creating black list: sd:/engage/config/DVC_List.txt");
        let filename = "sd:/engage/config/DVC_List.txt";
        let mut f = File::options().create(true).write(true).truncate(true).open(filename).unwrap();
        writeln!(&mut f, "* These are Items IDs that are removed from the Item randomization *").unwrap();
        for x in BLACKLIST_ITEMS {
            writeln!(&mut f, "remove_item\t{}", x).unwrap();
        }
        writeln!(&mut f, "\n* These are skill IDS that are removed from the skill randomization pool *").unwrap();
        for x in BLACKLIST_SKILL {
            writeln!(&mut f, "remove_skill\t{}", x).unwrap();
        }
        writeln!(&mut f, "\n* Following line is to remove the following skill IDs in the personal skill pool *").unwrap();
        let mut string = "remove_personal_skill".into();
        for x in PERSONAL_BLACK_LIST {
            string = format!("{}\t{}", string, x);
        }
        writeln!(&mut f, "{}\n", string).unwrap();
        writeln!(&mut f, "* Add Skill IDs in the Engage Skill pool for engage skill randomization pool here*").unwrap();
        writeln!(&mut f, "* Add all usable Skill IDs to Engage Skill pool by using 'add_sync_skill\tchaos*").unwrap();
        writeln!(&mut f, "add_engage_skill\tSID_example_skill_here_1\tSID_example_skill_here_2\n").unwrap();
        writeln!(&mut f, "* Add Skill IDs to Sync Skill pool, using 'add_sync_skill' for emblem sync skill randomization pool here *").unwrap();
        writeln!(&mut f, "* Add all usable Skill IDs to Sync Skill pool by using 'add_sync_skill\tchaos' *").unwrap();
        writeln!(&mut f, "add_sync_skill\tSID_example_skill_here_1\tSID_example_skill_here_2\n").unwrap();
    }
    println!("Total of {} Skills in Draconic Vibe Crystal Black List", SKILL_BLACK_LIST.lock().unwrap().len());
    println!("Total of {} Personal Skills in Draconic Vibe Crystal Black List", PERSONAL_BLIST.lock().unwrap().len());
    println!("Total of {} Item in Draconic Vibe Crystal Black List", ITEM_BLACK_LIST.lock().unwrap().len());
}

pub fn get_added_skills() {
    crate::randomizer::emblem::emblem_skill::ADDED_ENGAGE.lock().unwrap().clear();
    crate::randomizer::emblem::emblem_skill::ADDED_SYNC.lock().unwrap().clear();
    if let Ok(lines) = read_lines("sd:/engage/config/DVC_List.txt") {
        println!("Reading from sd:/engage/config/DVC_List.txt for added sync/engage skills.");
        for line in lines.flatten() {
            let spilt: Vec<_> = line.split_whitespace().collect();
            if spilt.len() > 1 {
                if spilt[0] == "*" || spilt[0] == "--" { continue; }    // comments
                match spilt[0] {
                    "add_engage_skill" => {                    
                        for z in 1..spilt.len() {
                            if is_valid_skill(spilt[z]){
                                let skill = SkillData::get(&spilt[z]).unwrap();
                                crate::randomizer::emblem::emblem_skill::ADDED_ENGAGE.lock().unwrap().push(skill.parent.index);
                                println!("Added to Engage Skill Pool: #{} {}", skill.parent.index, crate::utils::get_skill_name(skill));
                            }
                        } 
                    },
                    "add_sync_skill" => { 
                        for z in 1..spilt.len() {
                            if is_valid_skill(spilt[z]){
                                let skill = SkillData::get(&spilt[z]).unwrap();
                                crate::randomizer::emblem::emblem_skill::ADDED_SYNC.lock().unwrap().push(skill.parent.index);
                                println!("Added to Sync Skill Pool: #{} {}", skill.parent.index, crate::utils::get_skill_name(skill));
                            }
                        }
                    },
                    _ => { },
                }
            }
        }
    }
}