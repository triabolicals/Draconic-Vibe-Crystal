use super::*;

pub fn random_engrave_by_setting(setting: i32, loaded: bool) {
    if loaded && setting == 0 { return; }
    let data = get_data_read();
    match setting & 255 {
        1 => { randomize_engrave(data,0, 25); }
        2 => { randomize_engrave(data,-25, 25); }
        3 => { randomize_engrave(data,-50, 50); }
        4 => { randomize_engrave(data,-75, 75); }
        _ => { randomize_engrave(data, 0, 0); },
    }
}

pub fn randomize_engrave(data: &GameData, low: i32, high: i32) {
    if low == high {
        data.emblem_pool.emblem_data.iter().for_each(|x| {
            let god = x.get_god();
            x.engrave_stats.iter().enumerate().for_each(|(i, y)| { god.set_engrave_value(i as i32, *y) })
        });
    }
    else {
        let mut n_high = max(high, low);
        let mut n_low = min(low, high);
        if n_low > n_high - 30 { n_low = n_high - 30; }
        let pwr = [-5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, -1, -2, 2, 1];
        let weights = [-5, 5, 10, 4, -4, 3, -3, -2, -2, 2, 2, -1, -1, -1, 1, 1, 1, 0, 0, 15];
        
        let rng = get_rng();
        let mut engrave_stats: [i32; 6] = [0; 6];
        data.emblem_pool.emblem_data.iter().map(|x| x.get_god_mut()).for_each(|x|{
            let mut total = 0;
            let mut count = 0;
            while count < 20 {
                loop {
                    engrave_stats[0] = pwr[rng.get_value(15) as usize];
                    engrave_stats[1] = weights[rng.get_value(20) as usize];
                    if (engrave_stats[1]  + 2*engrave_stats[0]) >= n_low && (engrave_stats[1] + 5*engrave_stats[0]) <= n_high { break; }
                }
                loop {
                    engrave_stats[2]  = rng.get_value(14) - 5;
                    engrave_stats[3] = rng.get_value(14) - 5;
                    let combine = engrave_stats[2]  + engrave_stats[3];
                    if 5*combine >= n_low && 5*combine <= n_high { break; }
                }
                loop {
                    engrave_stats[4]  = rng.get_value(14) - 5;
                    engrave_stats[5] = rng.get_value(14) - 5;
                    let combine = engrave_stats[4]  + engrave_stats[5];
                    if 5*combine >= n_low && 5*combine <= n_high { break; }
                }
                total = ( engrave_stats[0]*2 + engrave_stats[2] + engrave_stats[3] + engrave_stats[4] + engrave_stats[5]) *5 - engrave_stats[1]*2;
                if total <= n_high && total >= n_low {
                    x.engrave_power = engrave_stats[0] as i8;
                    x.engrave_weight = engrave_stats[1] as i8;
                    x.engrave_hit = 5*engrave_stats[2] as i8;
                    x.engrave_avoid = 5*engrave_stats[3] as i8;
                    x.engrave_critical = 5*engrave_stats[4] as i8;
                    x.engrave_secure = 5*engrave_stats[5] as i8;
                    break;
                }
                count+= 1;
            }
        });
    }
}