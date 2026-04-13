pub const ALEAR: usize = 0;
pub const YUNAKA: usize = 10;
pub const SEADALL: usize = 26;
pub const VEYLE: usize = 32;
pub const MAUVIER: usize = 33;

pub const MPIDS: [&str;41] = [
    "MPID_Lueur", "MPID_Vandre", "MPID_Clan", "MPID_Fram", "MPID_Alfred", "MPID_Etie", "MPID_Boucheron", "MPID_Celine",
    "MPID_Chloe", "MPID_Louis", "MPID_Yunaka", "MPID_Staluke", "MPID_Citrinica", "MPID_Lapis", "MPID_Diamand", "MPID_Umber",
    "MPID_Jade", "MPID_Ivy", "MPID_Kagetsu", "MPID_Zelkova", "MPID_Fogato", "MPID_Pandoro", "MPID_Bonet", "MPID_Misutira",
    "MPID_Panetone", "MPID_Merin", "MPID_Hortensia", "MPID_Seadas", "MPID_Rosado", "MPID_Goldmary", "MPID_Linden", "MPID_Saphir",
    "MPID_Veyre", "MPID_Mauve", "MPID_Anna", "MPID_Jean", "MPID_El", "MPID_Rafale", "MPID_Selestia", "MPID_Gregory",
    "MPID_Madeline"
];

pub const PIDS: [&str; 41] = [
    "PID_リュール", "PID_ヴァンドレ", "PID_クラン", "PID_フラン", "PID_アルフレッド", "PID_エーティエ", "PID_ブシュロン", "PID_セリーヌ",
    "PID_クロエ", "PID_ルイ", "PID_ユナカ", "PID_スタルーク", "PID_シトリニカ", "PID_ラピス", "PID_ディアマンド", "PID_アンバー",
    "PID_ジェーデ", "PID_アイビー", "PID_カゲツ", "PID_ゼルコバ", "PID_フォガート", "PID_パンドロ", "PID_ボネ", "PID_ミスティラ",
    "PID_パネトネ", "PID_メリン", "PID_オルテンシア", "PID_セアダス", "PID_ロサード", "PID_ゴルドマリー", "PID_リンデン",
    "PID_ザフィーア", "PID_ヴェイル", "PID_モーヴ", "PID_アンナ", "PID_ジャン", "PID_エル", "PID_ラファール", "PID_セレスティア",
    "PID_グレゴリー", "PID_マデリーン"
];
pub const RECRUIT_CID: [&str; 41] = [
    "M001", "M001", "M002", "M002", "M003", "M003", "M003", "M004", "M004", "M004", "M006", "M007", "M007",
    "M007", "M008", "M008", "M009", "M011", "M011", "M011", "M012", "M012", "M012", "M013", "M013", "M013",
    "M014", "M015", "M016", "M016", "M018", "M019", "M022", "M021", "S002", "S001", "E006", "E006", "E006",
    "E006", "E006"
];
// Emblem and Emblem SKill Related
pub const RR_ORDER: [u8; 41] = [32, 33, 31, 30, 29, 28, 27, 26, 25, 24, 22, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 34, 10, 35, 9, 8, 7, 6, 5, 4, 3, 2, 0, 1, 21, 23, 36, 37, 38, 39, 40];
pub const EMBLEM_ASSET: [&str; 24] = ["マルス", "シグルド", "セリカ", "ミカヤ", "ロイ", "リーフ", "ルキナ", "リン", "アイク", "ベレト", "カムイ", "エイリーク", "エーデルガルト", "チキ", "ヘクトル", "ヴェロニカ", "セネリオ", "カミラ", "クロム", "リュール", "ディミトリ", "クロード", "ルフレ", "エフラム"];
pub const EIRIKA_TWIN_SKILLS: [&str; 12] = [ "SID_月の腕輪", "SID_太陽の腕輪", "SID_日月の腕輪", "SID_優風", "SID_勇空", "SID_蒼穹", "SID_月の腕輪＋", "SID_太陽の腕輪＋", "SID_日月の腕輪＋", "SID_優風＋", "SID_勇空＋", "SID_蒼穹＋" ];
pub const EMBLEM_GIDS: [&str; 24] = [
    "GID_マルス", "GID_シグルド", "GID_セリカ", "GID_ミカヤ", "GID_ロイ", "GID_リーフ", "GID_ルキナ", "GID_リン", "GID_アイク",
    "GID_ベレト", "GID_カムイ", "GID_エイリーク", "GID_エーデルガルト", "GID_チキ", "GID_ヘクトル", "GID_ヴェロニカ", "GID_セネリオ",
    "GID_カミラ", "GID_クロム", "GID_リュール", "GID_ディミトリ", "GID_クロード", "GID_ルフレ", "GID_エフラム"];

pub const RINGS: [&str; 23] = ["Marth", "Siglud", "Celica", "Micaiah", "Roy", "Leaf", "Lucina", "Lin", "Ike", "Byleth", "Kamui", "Eirik", "Edelgard", "Tiki", "Hector", "Veronica", "Senerio", "Camilla", "Chrom", "Dimitri", "Claude", "Reflet", "Ephraim"];
pub const EMBLEM_PARA: [&str; 19] = ["S014", "S009", "S013", "S011", "S012", "S010", "S003", "S004", "S005", "S006", "S007", "S008", "G007", "G001", "G002", "G003", "G004", "G005", "G006"];
pub const UNLOCK_PARA: [&str; 19] = ["M022", "M017", "M020", "M019", "M019", "M017", "M011", "M012", "M013", "M014", "M015", "M016", "", "G001", "G002", "G003", "G004", "G005", "G006"];
pub const PARA_LEVEL: [i32; 19] = [35, 28, 32, 28, 31, 30, 21, 22, 25, 28, 26, 29, -1, -1, -1, -1, -1, -1, -1];
pub const ENGAGE_ATK_AI: [&str; 11] = ["AI_AT_EngageAttack","AI_AT_EngagePierce", "AI_AT_EngageVision", "AI_AT_EngageDance", "AI_AT_EngageOverlap", "AI_AT_EngageBless", "AI_AT_EngageWaitGaze", "AI_AT_EngageSummon", "AI_AT_EngageCamilla", "AI_AT_Versus", "AI_AT_EngageWait"];
// Items

// Emblem Cutscene Related
pub const KENGEN: [&str; 19] = [
    "Scene03", "Kengen01", "Kengen02", "Kengen03", "Kengen04", "Kengen05", "Kengen07",
    "Kengen06", "Kengen08", "Kengen09", "Kengen10", "Kengen11", "Kengen13", "Kengen14",
    "Kengen15", "Kengen19", "Kengen16", "Kengen17", "Kengen18"
];

pub const ENGAGE_PREFIX: [&str; 29] = [
    "Mar", "Sig", "Cel", "Mic", "Roy", "Lei", "Luc", "Lyn", "Ike", "Byl", "Cor", "Eir",
    "Thr", "Tik", "Hec", "Ver", "Sor", "Cmi", "Chr", "Ler", "Ede", "Dim", "Cla", "Eph",
    "Enb", "Wng2D", "Cav2B", "Com0A", "Com0B"
];