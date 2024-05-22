use engage::gamedata::{*, skill::*, item::ItemData};
use std::sync::Mutex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::io::Write;

pub static SKILL_BLACK_LIST: Mutex<Vec<String>> = Mutex::new(Vec::new()); 
pub static ITEM_BLACK_LIST: Mutex<Vec<String>> = Mutex::new(Vec::new());
pub static PERSONAL_BLIST: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub const PIDS: [&str; 41] = ["PID_リュール", "PID_ヴァンドレ", "PID_クラン", "PID_フラン", "PID_アルフレッド", "PID_エーティエ", "PID_ブシュロン", "PID_セリーヌ", "PID_クロエ", "PID_ルイ", "PID_ユナカ", "PID_スタルーク", "PID_シトリニカ", "PID_ラピス", "PID_ディアマンド", "PID_アンバー", "PID_ジェーデ", "PID_アイビー", "PID_カゲツ", "PID_ゼルコバ", "PID_フォガート", "PID_パンドロ", "PID_ボネ", "PID_ミスティラ", "PID_パネトネ", "PID_メリン", "PID_オルテンシア", "PID_セアダス", "PID_ロサード", "PID_ゴルドマリー", "PID_リンデン", "PID_ザフィーア", "PID_ヴェイル", "PID_モーヴ", "PID_アンナ", "PID_ジャン", "PID_エル", "PID_ラファール", "PID_セレスティア", "PID_グレゴリー", "PID_マデリーン"];
pub const RECRUIT_CID : [&str; 41] = ["M001", "M001", "M002", "M002", "M003", "M003", "M003", "M004", "M004", "M004", "M006", "M007", "M007", "M007", "M008", "M008", "M009", "M011", "M011", "M011", "M012", "M012", "M012", "M013", "M013", "M013", "M014", "M015", "M016", "M016", "M018", "M019", "M022", "M021", "S002", "S001", "E006", "E006", "E006", "E006", "E006"];

// Emblem and Emblem SKill Related
pub const EMBLEM_ASSET: [&str; 22] = ["マルス", "シグルド", "セリカ", "ミカヤ", "ロイ", "リーフ", "ルキナ", "リン", "アイク", "ベレト", "カムイ", "エイリーク", "エーデルガルト", "チキ", "ヘクトル", "ヴェロニカ", "セネリオ", "カミラ", "クロム", "リュール", "ディミトリ", "クロード"];
pub const EIRIKA_TWIN_SKILLS: [&str; 12] = [ "SID_月の腕輪", "SID_太陽の腕輪", "SID_日月の腕輪", "SID_優風", "SID_勇空", "SID_蒼穹", "SID_月の腕輪＋", "SID_太陽の腕輪＋", "SID_日月の腕輪＋", "SID_優風＋", "SID_勇空＋", "SID_蒼穹＋" ];
pub const EMBLEM_GIDS: &[&str] = &["GID_マルス", "GID_シグルド", "GID_セリカ", "GID_ミカヤ", "GID_ロイ", "GID_リーフ", "GID_ルキナ", "GID_リン", "GID_アイク", "GID_ベレト", "GID_カムイ", "GID_エイリーク", "GID_エーデルガルト", "GID_チキ", "GID_ヘクトル", "GID_ヴェロニカ", "GID_セネリオ", "GID_カミラ", "GID_クロム"];
pub const RINGS: [&str; 19] = ["Marth", "Siglud", "Celica", "Micaiah", "Roy", "Leaf", "Lucina", "Lin", "Ike", "Byleth", "Kamui", "Eirik", "Edelgard", "Tiki", "Hector", "Veronica", "Senerio", "Camilla", "Chrom" ];
pub const EMBELM_PARA: [&str; 19] = ["S014", "S009", "S013", "S011", "S012", "S010", "S003", "S004", "S005", "S006", "S007", "S008", "G007", "G001", "G002", "G003", "G004", "G005", "G006"];
pub const UNLOCK_PARA: [&str; 19] = ["M022", "M017", "M020", "M019", "M019", "M017", "M011", "M012", "M013", "M014", "M015", "M016", "", "G001", "G002", "G003", "G004", "G005", "G006"];

// Items
pub const BLACKLIST_ITEMS: [&str; 25] = [
    "IID_マスタープルフ", "IID_リベラシオン改", "IID_リベラシオン改_ノーマル",
    "IID_リベラシオン", "IID_リベラシオン_M000", "IID_無し", "IID_不明", "IID_エンゲージ枠", "IID_火炎砲台", "IID_牙", "IID_邪竜石_E",
    "IID_邪竜石_E005", "IID_邪竜石_魔法攻撃_E", "IID_イル_反撃", "IID_イル_薙払いビーム", "IID_イル_突進",
    "IID_イル_吸収", "IID_イル_召喚", "IID_火のブレス", "IID_炎塊", "IID_ソンブル_物理攻撃",
    "IID_ソンブル_魔法攻撃", "IID_ソンブル_回転アタック", "IID_ソンブル_ビーム", "IID_ソンブル_エンゲージブレイク", 
];

// Skill Related
pub const STYLE: [&str;9] = ["Default", "Cooperation", "Horse", "Covert", "Heavy", "Fly", "Magic","Prana", "Dragon"];

pub const BLACKLIST_SKILL: &[&str] = &[
    "SID_バリア１", "SID_バリア２", "SID_バリア３", "SID_バリア４",
    "SID_バリア１_ノーマル用", "SID_バリア２_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア３_ノーマル用", "SID_バリア４_ノーマル用",
    "SID_異界の力_閉", "SID_異界の力_炎", "SID_異界の力_死", "SID_異界の力_夢", "SID_異界の力_科", "SID_守護者_E001", 
    "SID_守護者_E002", "SID_守護者_E003", "SID_守護者_E004", "SID_計略", "SID_無し", "SID_切磋琢磨", "SID_オルタネイト", "SID_双聖" ];

pub const MADDENING_BLACK_LIST: &[&str] = &[
    "SID_杖使い＋＋", "SID_杖使い＋", "SID_杖使い", "SID_残像", "SID_日月の腕輪", "SID_慈悲", "SID_計略_引込の計", "SID_計略_猛火計", "SID_計略_聖盾の備え", "SID_計略_毒矢", 
    "SID_守護者", "SID_守護者_使用不可", "SID_全弾発射", "SID_輸送隊", "SID_裏邪竜ノ子_兵種スキル", "SID_負けず嫌い", "SID_竜脈・異", "SID_先生", "SID_増幅_闇", "SID_重唱", "SID_大好物",
    "SID_熟練者", "SID_ブレイク無効", "SID_師の導き", "SID_拾得", "SID_竜脈", "SID_特別な踊り", "SID_契約", "SID_七色の叫び＋", "SID_七色の叫び", "SID_戦場の花", "SID_平和の花", "SID_大盤振る舞い", "SID_料理再現",
    "SID_一攫千金", "SID_努力の才", "SID_白の忠誠", "SID_碧の信仰", "SID_緋い声援", "SID_筋肉増強剤", "SID_虚無の呪い", "SID_自壊", "SID_自己回復", "SID_角の睨み", "SID_囮指名", "SID_戦技", "SID_血統", 
    "SID_引き寄せ", "SID_体当たり", "SID_入れ替え", "SID_異形狼連携", "SID_幻影狼連携", "SID_星玉の加護", "SID_双聖"
];

pub const PERSONAL_BLACK_LIST: &[&str] = &[ "SID_瘴気の領域", "SID_異形狼連携", "SID_幻影狼連携", "SID_全弾発射",  ];

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn generate_black_list() {
    for x in BLACKLIST_SKILL { SKILL_BLACK_LIST.lock().unwrap().push(x.to_string()); }
    for x in BLACKLIST_ITEMS { ITEM_BLACK_LIST.lock().unwrap().push(x.to_string()); }
    for x in PERSONAL_BLACK_LIST { PERSONAL_BLIST.lock().unwrap().push(x.to_string()); }

    if let Ok(lines) = read_lines("sd:/engage/config/DVC Blacklist.txt") {
        println!("Reading from sd:/engage/config/DVC Blacklist.txt");
        for line in lines.flatten() {
            let spilt: Vec<_> = line.split_whitespace().collect();
            if spilt.len() > 1 {
                if spilt[0] == "personal" || spilt[0] == "Personal" {
                    for z in 1..spilt.len() {
                        if spilt[z].contains("SID_") {
                            let skill = SkillData::get(&spilt[z]);
                            if skill.is_some() {
                                let sk = skill.unwrap();
                                if PERSONAL_BLIST.lock().unwrap().iter().find(|x| **x == spilt[z]).is_none() && SKILL_BLACK_LIST.lock().unwrap().iter().find(|x| **x == spilt[z]).is_none() {
                                    println!("Added to Personal Skill Blacklist #{}: {}", sk.parent.index, crate::utils::get_skill_name(sk));
                                    PERSONAL_BLIST.lock().unwrap().push(spilt[z].to_string());
                                }
                            }
                        } 
                    }
                    continue;
                }
                for z in 0..spilt.len() {
                    if spilt[z].contains("SID_") {
                        let skill = SkillData::get(&spilt[z]);
                        if skill.is_some() {
                            let sk = skill.unwrap();
                            if SKILL_BLACK_LIST.lock().unwrap().iter().find(|x| **x == spilt[z]).is_none(){
                                println!("Added General Skill Blacklist #{}: {}", sk.parent.index, crate::utils::get_skill_name(sk));
                                SKILL_BLACK_LIST.lock().unwrap().push(spilt[z].to_string());
                            }
                        } 
                    }
                    else if spilt[z].contains("IID_") {
                        let item = ItemData::get(&spilt[z]);
                        if item.is_some() {
                            let it = item.unwrap();
                            if ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == spilt[z]).is_none() {
                                println!("Added General Item Blacklist #{}: {}", it.parent.index, crate::utils::get_item_name(it));
                                ITEM_BLACK_LIST.lock().unwrap().push(spilt[z].to_string());
                            }
                        }
                    }
                }
            }
            else if line.contains("SID_") {
                let skill = SkillData::get(&line);
                if skill.is_some() {
                    let sk = skill.unwrap();
                    if SKILL_BLACK_LIST.lock().unwrap().iter().find(|x| **x == line).is_none(){
                        println!("Added General Skill Blacklist #{}: {}", sk.parent.index, crate::utils::get_skill_name(sk));
                        SKILL_BLACK_LIST.lock().unwrap().push(line.clone());
                    }
                }
            } 
            else if line.contains("IID_") {
                let item = ItemData::get(&line);
                if item.is_some() {
                    let it = item.unwrap();
                    if ITEM_BLACK_LIST.lock().unwrap().iter().find(|x| **x == line).is_none() {
                        println!("Added General Item Blacklist #{}: {}", it.parent.index, crate::utils::get_item_name(it));
                        ITEM_BLACK_LIST.lock().unwrap().push(line.clone());
                    }
                }
            }
        }
    }
    else {
        println!("Creating black list: sd:/engage/config/DVC Blacklist.txt");
        let filename = "sd:/engage/config/DVC Blacklist.txt";
        let mut f = File::options().create(true).write(true).truncate(true).open(filename).unwrap();
        for x in BLACKLIST_ITEMS {
            writeln!(&mut f, "{}", x).unwrap();
        }
        for x in BLACKLIST_SKILL {
            writeln!(&mut f, "{}", x).unwrap();
        }
        let mut string = "personal".into();
        for x in PERSONAL_BLACK_LIST {
            string = format!("{}\t{}", string, x);
        }
        writeln!(&mut f, "{}", string).unwrap();
    }
    println!("Total of {} Skills in Draconic Vibe Crystal Black List", SKILL_BLACK_LIST.lock().unwrap().len());
    println!("Total of {} Personal Skills in Black List", PERSONAL_BLIST.lock().unwrap().len());
    println!("Total of {} Item in Draconic Vibe Crystal Black List", ITEM_BLACK_LIST.lock().unwrap().len());
}

