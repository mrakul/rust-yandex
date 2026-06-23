use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Payload {
    // pub numbers: Vec<u32>,

    // Поменял на статический массив, ровно 10 чтобы влазило => process_json   time:   [301.55 ns 306.49 ns 311.56 ns]
    //                                                                        change: [-22.347% -19.667% -16.844%] (p = 0.00 < 0.05)
    //                                                                        Performance has improved.
    pub numbers: [u32; 10],

}

pub fn process_json(data: &str) -> Result<Payload, serde_json::Error> {
    serde_json::from_str(data)
}

pub fn sum_numbers(payload: &Payload) -> u64 {
    payload.numbers.iter().map(|&x| x as u64).sum()
} 