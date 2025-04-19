use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalPool {
    pub super_admin: Pubkey,         // 32
    pub loyalty_wallet: Pubkey,      // 32
    pub loyalty_fee: u64,            // 8
    pub total_round: u64,            // 8
    pub recent_players: Vec<Pubkey>, // 4 + 32 * 10
    pub recent_plays: Vec<GameData>, // 4 + 24 * 10
}

impl GlobalPool {
    pub const DATA_SIZE: usize = 32 + 32 + 8 + 8 + 4 + 24 * 10; //  324

    pub fn add_recent_play(&mut self, now: i64, reward: u64, token: u64, player: Pubkey) {
        if self.recent_players.len() == 10 {
            self.recent_players.pop();
        }
        if self.recent_plays.len() == 10 {
            self.recent_plays.pop();
        }

        self.recent_plays.insert(
            0,
            GameData {
                play_time: now,
                reward_amount: reward,
                token,
            },
        );
        self.recent_players.insert(0, player);
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct GameData {
    pub play_time: i64,     // 8
    pub reward_amount: u64, // 8
    pub token: u64,         // 8
}

impl GameData {
    pub const DATA_SIZE: usize = 8 + 8 + 8; // 24
}

#[account]
#[derive(Default)]
pub struct PlayerPool {
    // 104
    pub player: Pubkey,            // 32
    pub round: u64,                // 8
    pub game_data: GameData,       // 24
    pub win_times: u64,            // 8
    pub received_reward: u64,      // 8
    pub claimable_reward: u64,     // 8
    pub claimable_token: [u64; 9], // 8 * 9
}

impl PlayerPool {
    pub const DATA_SIZE: usize = 32 + 8 + 24 + 8 + 8 + 8 + 8 * 9; // 160

    pub fn add_game_data(&mut self, now: i64, reward: u64, token: u64) {
        self.game_data.play_time = now;
        self.game_data.reward_amount = reward;
        self.game_data.token = token;
        self.round += 1;
        if reward > 0 {
            self.win_times += 1;
            if token == 0 {
                self.received_reward += reward;
                self.claimable_reward += reward;
            } else {
                self.claimable_token[token as usize - 1] += reward;
            }
        }
    }
}
