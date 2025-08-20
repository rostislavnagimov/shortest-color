pub mod color;
pub use color::{convert_to_color, is_valid_color, shorten_color};

use std::sync::Mutex;
use std::time::{Duration, Instant};

static STATS: Mutex<BenchStats> = Mutex::new(BenchStats::new());

struct BenchStats {
    validation_times: Vec<Duration>,
    conversion_times: Vec<Duration>,
    shortening_times: Vec<Duration>,
    total_times: Vec<Duration>,
}

impl BenchStats {
    const fn new() -> Self {
        Self {
            validation_times: Vec::new(),
            conversion_times: Vec::new(),
            shortening_times: Vec::new(),
            total_times: Vec::new(),
        }
    }

    fn add_measurement(
        &mut self,
        validation: Duration,
        conversion: Duration,
        shortening: Option<Duration>,
        total: Duration,
    ) {
        self.validation_times.push(validation);
        self.conversion_times.push(conversion);
        if let Some(s) = shortening {
            self.shortening_times.push(s);
        }
        self.total_times.push(total);
    }

    fn print_averages(&self) {
        if self.validation_times.is_empty() {
            println!("No measurements recorded");
            return;
        }

        let avg_validation = self.average(&self.validation_times);
        let avg_conversion = self.average(&self.conversion_times);
        let avg_shortening = if !self.shortening_times.is_empty() {
            Some(self.average(&self.shortening_times))
        } else {
            None
        };
        let avg_total = self.average(&self.total_times);

        println!("\n📊 СРЕДНЯЯ ПРОИЗВОДИТЕЛЬНОСТЬ:");
        println!("   Валидация:   {:?}", avg_validation);
        println!("   Конверсия:   {:?}", avg_conversion);
        if let Some(avg_short) = avg_shortening {
            println!("   Сокращение:  {:?}", avg_short);
        }
        println!("   Общее время: {:?}", avg_total);
        println!("   Измерений:   {}", self.validation_times.len());
    }

    fn average(&self, times: &[Duration]) -> Duration {
        if times.is_empty() {
            return Duration::from_nanos(0);
        }
        
        let total_nanos: u128 = times.iter().map(|d| d.as_nanos()).sum();
        Duration::from_nanos((total_nanos / times.len() as u128) as u64)
    }
}

pub fn shorten_css_color(color_str: &str) -> String {
    let start = Instant::now();
    
    let validation_start = Instant::now();
    let is_valid = is_valid_color(color_str);
    let validation_time = validation_start.elapsed();
    
    if !is_valid {
        let total_time = start.elapsed();
        println!(
            "shorten_css_color('{}') -> '{}' | validation: {:?}, total: {:?}",
            color_str, color_str, validation_time, total_time
        );
        
        if let Ok(mut stats) = STATS.lock() {
            stats.add_measurement(validation_time, Duration::from_nanos(0), None, total_time);
        }
        
        return color_str.to_string();
    }
    
    let conversion_start = Instant::now();
    let result = if let Some(color) = convert_to_color(color_str) {
        let conversion_time = conversion_start.elapsed();
        let shortening_start = Instant::now();
        let shortened = shorten_color(&color);
        let shortening_time = shortening_start.elapsed();
        let total_time = start.elapsed();
        
        println!(
            "shorten_css_color('{}') -> '{}' | validation: {:?}, conversion: {:?}, shortening: {:?}, total: {:?}", 
            color_str, shortened, validation_time, conversion_time, shortening_time, total_time
        );
        
        if let Ok(mut stats) = STATS.lock() {
            stats.add_measurement(validation_time, conversion_time, Some(shortening_time), total_time);
        }
        
        shortened
    } else {
        let conversion_time = conversion_start.elapsed();
        let total_time = start.elapsed();
        
        println!(
            "shorten_css_color('{}') -> '{}' | validation: {:?}, conversion: {:?} (failed), total: {:?}", 
            color_str, color_str, validation_time, conversion_time, total_time
        );
        
        if let Ok(mut stats) = STATS.lock() {
            stats.add_measurement(validation_time, conversion_time, None, total_time);
        }
        
        color_str.to_string()
    };
    
    result
}

pub fn print_benchmark_summary() {
    if let Ok(stats) = STATS.lock() {
        stats.print_averages();
    } else {
        println!("Failed to acquire stats lock for summary");
    }
}

pub fn reset_benchmark_stats() {
    if let Ok(mut stats) = STATS.lock() {
        *stats = BenchStats::new();
    }
}