pub const CF_VERSION: &str = "0.3"; // hack proof

pub const GLOBAL_AUTHORITY_SEED: &str = "global-authority";
pub const VAULT_AUTHORITY_SEED: &str = "vault-authority";
pub const PLAYER_POOL_SEED: &str = "player-pool";

pub const MAX_NAME_LENGTH: usize = 24;

pub const LOYALTY_WALLET: &str = "JAk3U6ksWV7mULF2osVeL3XzSug1TGRXguAegVuj3CNt";
pub const LOYALTY_FEE: u64 = 40; // 4%
pub const PERMILLE: u64 = 1000;

pub const BET_SOL_AMOUNT: [u64; 6] = [
    50_000_000,
    100_000_000,
    250_000_000,
    500_000_000,
    1_000_000_000,
    2_000_000_000,
];
pub struct TokenInfo<'a> {
    pub name: &'a str,
    pub mint: &'a str,
    pub bet_amount: [u64; 6],
}

pub const TOKEN_INFO: [TokenInfo; 9] = [
    TokenInfo {
        name: "SOUL",
        mint: "F6weWmuc1vwdL4u38Ro9jKXHEMjP9BoNdWMF5o5TvtJf",
        bet_amount: [
            5_000_000_000,
            10_000_000_000,
            25_000_000_000,
            50_000_000_000,
            100_000_000_000,
            200_000_000_000,
        ],
    },
    TokenInfo {
        name: "NANA",
        mint: "HxRELUQfvvjToVbacjr9YECdfQMUqGgPYB68jVDYxkbr",
        bet_amount: [
            50_000_000_000,
            100_000_000_000,
            250_000_000_000,
            500_000_000_000,
            1000_000_000_000,
            2000_000_000_000,
        ],
    },
    TokenInfo {
        name: "COCO",
        mint: "74DSHnK1qqr4z1pXjLjPAVi8XFngZ635jEVpdkJtnizQ",
        bet_amount: [
            2500_00000,
            5000_00000,
            12500_00000,
            25000_00000,
            50000_00000,
            100000_00000,
        ],
    },
    TokenInfo {
        name: "ROYAL",
        mint: "D7rcV8SPxbv94s3kJETkrfMrWqHFs6qrmtbiu6saaany",
        bet_amount: [
            5000_00000,
            10000_00000,
            25000_00000,
            50000_00000,
            100000_00000,
            200000_00000,
        ],
    },
    TokenInfo {
        name: "GP",
        mint: "31k88G5Mq7ptbRDf3AM13HAq6wRQHXHikR8hik7wPygk",
        bet_amount: [
            2_000_000_000,
            4_000_000_000,
            10_000_000_000,
            20_000_000_000,
            40_000_000_000,
            80_000_000_000,
        ],
    },
    TokenInfo {
        name: "FORGE",
        mint: "FoRGERiW7odcCBGU1bztZi16osPBHjxharvDathL5eds",
        bet_amount: [
            10_000_000_000,
            20_000_000_000,
            50_000_000_000,
            100_000_000_000,
            200_000_000_000,
            400_000_000_000,
        ],
    },
    TokenInfo {
        name: "JELLY",
        mint: "9WMwGcY6TcbSfy9XPpQymY3qNEsvEaYL3wivdwPG2fpp",
        bet_amount: [
            5_000_000,
            10_000_000,
            25_000_000,
            50_000_000,
            100_000_000,
            200_000_000,
        ],
    },
    TokenInfo {
        name: "2080",
        mint: "Dwri1iuy5pDFf2u2GwwsH2MxjR6dATyDv9En9Jk8Fkof",
        bet_amount: [
            30_000_000_000,
            60_000_000_000,
            150_000_000_000,
            300_000_000_000,
            600_000_000_000,
            1200_000_000_000,
        ],
    },
    TokenInfo {
        name: "BOOTY",
        mint: "bootyAfCh1eSQeKhFaDjN9Pu6zwPmAoQPoJWVuPasjJ",
        bet_amount: [
            30_000_000_000,
            60_000_000_000,
            150_000_000_000,
            300_000_000_000,
            600_000_000_000,
            1200_000_000_000,
        ],
    },
];
