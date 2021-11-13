use crate::CollectedData;

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct ProcessedStatistics {
    pub max: u64,
    pub min: u64,
    pub avg: f64,
    pub percentile_50: u64,
    pub percentile_90: u64,
    pub percentile_99: u64,
}

impl<'a> From<&CollectedData> for ProcessedStatistics {
    fn from(data: &CollectedData) -> Self {
        let mut guard = data.data.lock().unwrap();

        guard.sort();

        let max = *guard.last().unwrap();
        let min = *guard.first().unwrap();

        let sum: u64 = guard.iter().sum();
        let count = guard.len();
        let avg = sum as f64 / count as f64;

        let percentile_50 = get_percentile(guard.as_slice(), 0.5);
        let percentile_90 = get_percentile(guard.as_slice(), 0.9);
        let percentile_99 = get_percentile(guard.as_slice(), 0.99);

        Self {
            max,
            min,
            avg,
            percentile_50,
            percentile_90,
            percentile_99,
        }
    }
}

fn get_percentile(data: &[u64], percentile: f64) -> u64 {
    let n = ((data.len() - 1) as f64 * percentile) as usize;
    data[n]
}
